#!/usr/bin/env bash
set -euo pipefail

VM_NAME="meal-planning-vm"
ZONE="us-east1-b"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

[[ -f "$PROJECT_ROOT/.env" ]] && set -a && source "$PROJECT_ROOT/.env" && set +a

info "Building release binary locally..."
cd "$PROJECT_ROOT"
cargo leptos build --release

BINARY=$(find "$PROJECT_ROOT/target" -maxdepth 3 -path "*/release/ai_meal_planning" -type f | head -1)
if [[ -z "$BINARY" ]]; then
    warn "No binary found in target/release — did the build succeed?"
    exit 1
fi

info "Checking if binary changed..."
LOCAL_HASH=$(sha256sum "$BINARY" | cut -d' ' -f1)
REMOTE_HASH=$(gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- \
    "cat ~/meal-planning/ai_meal_planning.sha256 2>/dev/null || echo ''" | tr -d '\r')

if [[ "$LOCAL_HASH" == "$REMOTE_HASH" ]]; then
    info "Binary unchanged, skipping copy."
else
    info "Stopping service..."
    gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- "sudo systemctl stop meal-planning 2>/dev/null || true"

    info "Copying binary to VM..."
    gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- "mkdir -p ~/meal-planning/target ~/meal-planning/data"
    gcloud compute scp \
        "$BINARY" \
        "$VM_NAME":~/meal-planning/ai_meal_planning \
        --zone="$ZONE" --quiet
    gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- \
        "chmod +x ~/meal-planning/ai_meal_planning && sudo chcon -t bin_t ~/meal-planning/ai_meal_planning && echo '$LOCAL_HASH' > ~/meal-planning/ai_meal_planning.sha256"
fi
gcloud compute scp --recurse \
    "$PROJECT_ROOT/target/site" \
    "$VM_NAME":~/meal-planning/target/ \
    --zone="$ZONE" --quiet
gcloud compute scp --recurse \
    "$PROJECT_ROOT/locales" \
    "$PROJECT_ROOT/style" \
    "$VM_NAME":~/meal-planning/ \
    --zone="$ZONE" --quiet

info "Generating self-signed certificate (if not already present)..."
gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- bash <<'CERTS'
command -v openssl &>/dev/null || sudo dnf install -y openssl
if [[ ! -f ~/meal-planning/certs/cert.pem ]]; then
    mkdir -p ~/meal-planning/certs
    openssl req -x509 -newkey rsa:4096 \
        -keyout ~/meal-planning/certs/key.pem \
        -out ~/meal-planning/certs/cert.pem \
        -days 365 -nodes \
        -subj "/CN=meal-planning"
    chmod 600 ~/meal-planning/certs/key.pem
    echo "Certificate generated."
else
    echo "Certificate already exists, skipping."
fi
CERTS

info "Installing/updating systemd service..."
VM_USER=$(gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- "echo \$USER" 2>/dev/null | tr -d '\r')
if [[ -z "${AUTH_PASSWORD:-}" ]]; then
    read -rsp "AUTH_PASSWORD: " AUTH_PASSWORD_VALUE
    echo
    if [[ -z "$AUTH_PASSWORD_VALUE" ]]; then
        warn "AUTH_PASSWORD is required."
        exit 1
    fi
else
    AUTH_PASSWORD_VALUE="$AUTH_PASSWORD"
fi

gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- bash <<SERVICE
sudo tee /etc/systemd/system/meal-planning.service > /dev/null <<EOF
[Unit]
Description=Meal Planning App
After=network.target

[Service]
Type=simple
User=$VM_USER
WorkingDirectory=/home/$VM_USER/meal-planning
ExecStart=/home/$VM_USER/meal-planning/ai_meal_planning
Environment=LEPTOS_OUTPUT_NAME=ai_meal_planning
Environment=LEPTOS_SITE_ROOT=target/site
Environment=LEPTOS_SITE_ADDR=0.0.0.0:3000
Environment=LEPTOS_ENV=PROD
Environment=DATABASE_URL=sqlite:///home/$VM_USER/meal-planning/data/meal_planning.db
Environment="AUTH_PASSWORD=$AUTH_PASSWORD_VALUE"
Environment=RUST_LOG=info
Environment=CERT_FILE=/home/$VM_USER/meal-planning/certs/cert.pem
Environment=KEY_FILE=/home/$VM_USER/meal-planning/certs/key.pem
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable --now meal-planning || sudo systemctl restart meal-planning
SERVICE

info "Update complete."
