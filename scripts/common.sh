# Sourced by scripts/*.sh. The VM is shared with wheel, so it and its project
# have neutral names. It has no external IP: ssh/scp go through IAP,
# outbound traffic through Cloud NAT, and the app is reached through an SSH
# tunnel (scripts/tunnel.sh). Sourcing makes no gcloud calls.

PROJECT_NAME="homelab"
VM_NAME="homelab-vm"
ZONE="us-east1-b"
REGION="us-east1"
NETWORK="default"
ROUTER="homelab-router"
NAT="homelab-nat"
IAP_RULE="homelab-allow-iap-ssh"
IAP_RANGE="35.235.240.0/20" # Google's IAP TCP-forwarding range

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# PROJECT_ID by display name; the active gcloud project is left alone.
resolve_project() {
    PROJECT_ID=$(gcloud projects list --filter="name='$PROJECT_NAME'" \
        --format="value(projectId)" 2>/dev/null | head -1 || true)
    if [[ -z "$PROJECT_ID" ]]; then
        warn "Could not find GCP project '$PROJECT_NAME' (is gcloud logged in? gcloud auth login)"
        exit 1
    fi
}

vm_ssh() { gcloud compute ssh "$VM_NAME" --zone="$ZONE" --project="$PROJECT_ID" --tunnel-through-iap --quiet -- "$@"; }
vm_scp() { gcloud compute scp --zone="$ZONE" --project="$PROJECT_ID" --tunnel-through-iap --quiet "$@"; }

# Outbound internet without an external IP (Cloud NAT), and SSH from IAP
# only. Safe to re-run.
ensure_network() {
    if ! gcloud compute routers describe "$ROUTER" --region="$REGION" --project="$PROJECT_ID" &>/dev/null; then
        info "Creating Cloud Router $ROUTER..."
        gcloud compute routers create "$ROUTER" --network="$NETWORK" --region="$REGION" --project="$PROJECT_ID"
    fi
    if ! gcloud compute routers nats describe "$NAT" --router="$ROUTER" --region="$REGION" \
        --project="$PROJECT_ID" &>/dev/null; then
        info "Creating Cloud NAT $NAT..."
        gcloud compute routers nats create "$NAT" --router="$ROUTER" --region="$REGION" \
            --project="$PROJECT_ID" --auto-allocate-nat-external-ips --nat-all-subnet-ip-ranges
    fi
    if ! gcloud compute firewall-rules describe "$IAP_RULE" --project="$PROJECT_ID" &>/dev/null; then
        info "Creating firewall rule $IAP_RULE (SSH from IAP only)..."
        gcloud compute firewall-rules create "$IAP_RULE" --project="$PROJECT_ID" --network="$NETWORK" \
            --direction=INGRESS --allow=tcp:22 --source-ranges="$IAP_RANGE" --description="SSH through IAP"
    fi
}
