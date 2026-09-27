#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/common.sh"

# To list available Fedora images run:
#   gcloud compute images list --project fedora-cloud --no-standard-images
FEDORA_IMAGE_PROJECT="fedora-cloud"
FEDORA_IMAGE_FAMILY="fedora-cloud-44-x86-64"

# ── GCP project ────────────────────────────────────────────────────────────────
PROJECT_ID=$(gcloud projects list --filter="name='$PROJECT_NAME'" --format="value(projectId)" 2>/dev/null | head -1 || true)

if [[ -n "$PROJECT_ID" ]]; then
  info "Found existing project '$PROJECT_NAME' ($PROJECT_ID), skipping setup."
else
  PROJECT_ID="$PROJECT_NAME"
  info "Creating project: $PROJECT_ID"
  if ! gcloud projects create "$PROJECT_ID" --name="$PROJECT_NAME"; then
    warn "Project ID '$PROJECT_ID' is taken. Set a unique PROJECT_ID here and retry."
    exit 1
  fi

  echo ""
  warn "Link a billing account before continuing (required even for the free tier)."
  echo "  1. gcloud billing accounts list"
  echo "  2. gcloud billing projects link $PROJECT_ID --billing-account=YOUR_BILLING_ID"
  echo ""
  read -rp "Press Enter once billing is linked..."
fi
gcloud services enable compute.googleapis.com iap.googleapis.com --project="$PROJECT_ID"

# ── VM: no external IP; NAT out, IAP in ────────────────────────────────────────
info "Creating Fedora VM (e2-micro, free tier, no external IP)..."
gcloud compute instances create "$VM_NAME" \
  --project="$PROJECT_ID" \
  --machine-type=e2-micro \
  --zone="$ZONE" \
  --image-family="$FEDORA_IMAGE_FAMILY" \
  --image-project="$FEDORA_IMAGE_PROJECT" \
  --boot-disk-size=20GB \
  --boot-disk-type=pd-standard \
  --no-address
ensure_network

# ── Build locally ──────────────────────────────────────────────────────────────
info "Building release binary locally..."
cd "$PROJECT_ROOT"
cargo leptos build --release

# ── Deploy ─────────────────────────────────────────────────────────────────────
info "Waiting for SSH to become available..."
for _ in $(seq 1 12); do
  vm_ssh "echo ok" 2>/dev/null && break
  sleep 10
done

info "Preparing app directory on VM..."
vm_ssh "mkdir -p ~/meal-planning/target ~/meal-planning/data"

info "Copying binary and assets..."
BINARY=$(find "$PROJECT_ROOT/target" -maxdepth 3 -path "*/release/ai_meal_planning" -type f | head -1)
if [[ -z "$BINARY" ]]; then
    warn "No binary found in target/release — did the build succeed?"
    exit 1
fi
info "Found binary: $BINARY"
vm_scp "$BINARY" "$VM_NAME":~/meal-planning/ai_meal_planning
vm_scp --recurse "$PROJECT_ROOT/target/site" "$VM_NAME":~/meal-planning/target/
vm_scp --recurse "$PROJECT_ROOT/locales" "$PROJECT_ROOT/style" "$VM_NAME":~/meal-planning/

# ── TLS certificates ───────────────────────────────────────────────────────────
info "Generating self-signed certificate (if not already present)..."
vm_ssh bash <<'CERTS'
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
vm_ssh "chmod +x ~/meal-planning/ai_meal_planning && sudo chcon -t bin_t ~/meal-planning/ai_meal_planning"

info "Installing systemd service..."
VM_USER=$(vm_ssh "echo \$USER" 2>/dev/null | tr -d '\r')

vm_ssh bash <<SERVICE
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

echo ""
info "Done! Reach it: scripts/tunnel.sh, then https://localhost:3000"
warn "Browser will warn about the self-signed certificate — this is expected."
warn "Change the default password: run scripts/update.sh with AUTH_PASSWORD in .env"
