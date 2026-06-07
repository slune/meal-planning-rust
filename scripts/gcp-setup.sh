#!/usr/bin/env bash
set -euo pipefail

PROJECT_NAME="meal-planning"
VM_NAME="meal-planning-vm"
ZONE="us-east1-b"

# To list available Fedora images run:
#   gcloud compute images list --project fedora-cloud --no-standard-images
FEDORA_IMAGE_PROJECT="fedora-cloud"
FEDORA_IMAGE_FAMILY="fedora-cloud-44-x86-64"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── GCP project ────────────────────────────────────────────────────────────────
PROJECT_ID=$(gcloud projects list --filter="name='$PROJECT_NAME'" --format="value(projectId)" 2>/dev/null | head -1)

if [[ -n "$PROJECT_ID" ]]; then
  info "Found existing project '$PROJECT_NAME' ($PROJECT_ID), skipping setup."
else
  PROJECT_ID="meal-planning"
  info "Creating project: $PROJECT_ID"
  if ! gcloud projects create "$PROJECT_ID" --name="$PROJECT_NAME"; then
    warn "Project ID '$PROJECT_ID' is taken. Set a unique PROJECT_ID at the top of this script and retry."
    exit 1
  fi

  echo ""
  warn "Link a billing account before continuing (required even for the free tier)."
  echo "  1. gcloud billing accounts list"
  echo "  2. gcloud billing projects link $PROJECT_ID --billing-account=YOUR_BILLING_ID"
  echo ""
  read -rp "Press Enter once billing is linked..."

  gcloud services enable compute.googleapis.com
  info "Compute Engine API enabled."
fi
gcloud config set project "$PROJECT_ID"

# ── VM ─────────────────────────────────────────────────────────────────────────
info "Creating Fedora VM (e2-micro, free tier)..."
gcloud compute instances create "$VM_NAME" \
  --machine-type=e2-micro \
  --zone="$ZONE" \
  --image-family="$FEDORA_IMAGE_FAMILY" \
  --image-project="$FEDORA_IMAGE_PROJECT" \
  --boot-disk-size=20GB \
  --boot-disk-type=pd-standard \
  --tags=meal-planning

gcloud compute firewall-rules create meal-planning-allow-app \
  --allow=tcp:3000 \
  --target-tags=meal-planning \
  --source-ranges=0.0.0.0/0 \
  --description="Meal planning app" 2>/dev/null ||
  warn "Firewall rule already exists, skipping."

# ── Build locally ──────────────────────────────────────────────────────────────
info "Building release binary locally..."
cd "$PROJECT_ROOT"
cargo leptos build --release

# ── Deploy ─────────────────────────────────────────────────────────────────────
info "Waiting for SSH to become available..."
for i in $(seq 1 12); do
  gcloud compute ssh "$VM_NAME" --zone="$ZONE" --quiet -- "echo ok" 2>/dev/null && break
  sleep 10
done

info "Preparing app directory on VM..."
gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- "mkdir -p ~/meal-planning/target ~/meal-planning/data"

info "Copying binary and assets..."
BINARY=$(find "$PROJECT_ROOT/target" -maxdepth 3 -path "*/release/ai_meal_planning" -type f | head -1)
if [[ -z "$BINARY" ]]; then
    warn "No binary found in target/release — did the build succeed?"
    exit 1
fi
info "Found binary: $BINARY"
gcloud compute scp \
  "$BINARY" \
  "$VM_NAME":~/meal-planning/ai_meal_planning \
  --zone="$ZONE" --quiet
gcloud compute scp --recurse \
  "$PROJECT_ROOT/target/site" \
  "$VM_NAME":~/meal-planning/target/ \
  --zone="$ZONE" --quiet
gcloud compute scp --recurse \
  "$PROJECT_ROOT/locales" \
  "$PROJECT_ROOT/style" \
  "$VM_NAME":~/meal-planning/ \
  --zone="$ZONE" --quiet

# ── TLS certificates ───────────────────────────────────────────────────────────
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

# ── Systemd service ────────────────────────────────────────────────────────────
gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- "chmod +x ~/meal-planning/ai_meal_planning && sudo chcon -t bin_t ~/meal-planning/ai_meal_planning"

info "Installing systemd service..."
VM_USER=$(gcloud compute ssh "$VM_NAME" --zone="$ZONE" -- "echo \$USER" 2>/dev/null | tr -d '\r')

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
Environment=AUTH_PASSWORD=admin123
Environment=RUST_LOG=info
Environment=CERT_FILE=/home/$VM_USER/meal-planning/certs/cert.pem
Environment=KEY_FILE=/home/$VM_USER/meal-planning/certs/key.pem
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable --now meal-planning
SERVICE

EXTERNAL_IP=$(gcloud compute instances describe "$VM_NAME" \
  --zone="$ZONE" \
  --format="get(networkInterfaces[0].accessConfigs[0].natIP)")

echo ""
info "Done! App running at: https://${EXTERNAL_IP}:3000"
warn "Browser will warn about the self-signed certificate — this is expected."
warn "Change the default password: edit AUTH_PASSWORD in /etc/systemd/system/meal-planning.service then sudo systemctl restart meal-planning"
