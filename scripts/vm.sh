#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-}"
VM_NAME="meal-planning-vm"
ZONE="us-east1-b"
REGION="us-east1"
STATIC_IP_NAME="meal-planning-ip"
FIREWALL_RULE="meal-planning-allow-app"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
[[ -f "$SCRIPT_DIR/../.env" ]] && set -a && source "$SCRIPT_DIR/../.env" && set +a

PROJECT_ID=$(gcloud projects list --filter="name='meal-planning'" --format="value(projectId)" 2>/dev/null | head -1)
if [[ -z "$PROJECT_ID" ]]; then
    warn "Could not find GCP project 'meal-planning'."
    exit 1
fi
gcloud config set project "$PROJECT_ID" --quiet

vm_status() {
    gcloud compute instances describe "$VM_NAME" --zone="$ZONE" \
        --format="value(status)" 2>/dev/null || echo "NOT_FOUND"
}

# Block until the VM settles into a stable state (RUNNING / TERMINATED /
# NOT_FOUND), so we never issue a start/stop against a transitional VM.
wait_for_stable() {
    local status
    for _ in $(seq 1 60); do
        status=$(vm_status)
        case "$status" in
            RUNNING|TERMINATED|SUSPENDED|NOT_FOUND) echo "$status"; return 0 ;;
        esac
        warn "VM is in transitional state '$status', waiting..."
        sleep 5
    done
    echo "$status"
}

case "$ACTION" in
    start)
        # ── Preconditions ───────────────────────────────────────────────────
        if [[ "$(wait_for_stable)" == "NOT_FOUND" ]]; then
            warn "VM '$VM_NAME' does not exist in zone '$ZONE'. Create it first."
            exit 1
        fi

        # ── Static IP ───────────────────────────────────────────────────────
        if ! gcloud compute addresses describe "$STATIC_IP_NAME" --region="$REGION" &>/dev/null; then
            info "Reserving static IP..."
            gcloud compute addresses create "$STATIC_IP_NAME" --region="$REGION"
        else
            info "Static IP already reserved."
        fi
        STATIC_IP=$(gcloud compute addresses describe "$STATIC_IP_NAME" \
            --region="$REGION" --format="value(address)")

        # ── Firewall ────────────────────────────────────────────────────────
        if ! gcloud compute firewall-rules describe "$FIREWALL_RULE" &>/dev/null; then
            info "Creating firewall rule..."
            gcloud compute firewall-rules create "$FIREWALL_RULE" \
                --allow=tcp:3000 \
                --target-tags=meal-planning \
                --source-ranges=0.0.0.0/0 \
                --description="Meal planning app"
        else
            info "Firewall rule already exists."
        fi

        # Assign static IP to VM if not already attached.
        # GCP only allows modifying access configs on a stopped VM, and a NIC
        # can hold at most one — so we must delete the existing config (whatever
        # it's named) before adding ours.
        CURRENT_IP=$(gcloud compute instances describe "$VM_NAME" --zone="$ZONE" \
            --format="value(networkInterfaces[0].accessConfigs[0].natIP)" 2>/dev/null || echo "")
        if [[ "$CURRENT_IP" != "$STATIC_IP" ]]; then
            if [[ "$(vm_status)" == "RUNNING" ]]; then
                info "Stopping VM to reassign static IP..."
                gcloud compute instances stop "$VM_NAME" --zone="$ZONE"
                wait_for_stable >/dev/null
            fi
            info "Assigning static IP $STATIC_IP to VM..."
            # Look up the existing access config's actual name (e.g. "external-nat"
            # vs "External NAT") so the delete reliably removes it.
            EXISTING_AC=$(gcloud compute instances describe "$VM_NAME" --zone="$ZONE" \
                --format="value(networkInterfaces[0].accessConfigs[0].name)" 2>/dev/null || echo "")
            if [[ -n "$EXISTING_AC" ]]; then
                gcloud compute instances delete-access-config "$VM_NAME" \
                    --zone="$ZONE" --access-config-name="$EXISTING_AC"
            fi
            gcloud compute instances add-access-config "$VM_NAME" \
                --zone="$ZONE" --access-config-name="external-nat" \
                --address="$STATIC_IP"
        else
            info "Static IP already attached to VM."
        fi

        # ── Start VM ────────────────────────────────────────────────────────
        if [[ "$(vm_status)" == "RUNNING" ]]; then
            info "VM is already running."
        else
            info "Starting VM..."
            gcloud compute instances start "$VM_NAME" --zone="$ZONE"
        fi

        echo ""
        info "App available at: https://${STATIC_IP}:3000"
        ;;

    stop)
        # ── Stop VM ─────────────────────────────────────────────────────────
        STATUS=$(wait_for_stable)
        if [[ "$STATUS" == "NOT_FOUND" ]]; then
            info "VM does not exist; nothing to stop."
        elif [[ "$STATUS" == "TERMINATED" ]]; then
            info "VM is already stopped."
        else
            info "Stopping VM..."
            gcloud compute instances stop "$VM_NAME" --zone="$ZONE"
        fi

        # ── Release static IP ───────────────────────────────────────────────
        if gcloud compute addresses describe "$STATIC_IP_NAME" --region="$REGION" &>/dev/null; then
            info "Releasing static IP (reserved IPs are billed when not in use)..."
            gcloud compute addresses delete "$STATIC_IP_NAME" --region="$REGION" --quiet
        else
            info "No static IP to release."
        fi

        echo ""
        info "VM stopped. Only the boot disk remains billable (~\$0.04/month)."
        ;;

    destroy)
        echo ""
        warn "This will permanently delete the VM, its boot disk, the static IP, and the firewall rule."
        read -rp "Type 'yes' to confirm: " confirm
        [[ "$confirm" == "yes" ]] || { echo "Aborted."; exit 1; }

        if [[ "$(wait_for_stable)" == "NOT_FOUND" ]]; then
            info "VM already deleted."
        else
            info "Deleting VM and boot disk..."
            gcloud compute instances delete "$VM_NAME" --zone="$ZONE" --delete-disks=all --quiet
        fi

        if gcloud compute addresses describe "$STATIC_IP_NAME" --region="$REGION" &>/dev/null; then
            info "Releasing static IP..."
            gcloud compute addresses delete "$STATIC_IP_NAME" --region="$REGION" --quiet
        fi

        if gcloud compute firewall-rules describe "$FIREWALL_RULE" &>/dev/null; then
            info "Deleting firewall rule..."
            gcloud compute firewall-rules delete "$FIREWALL_RULE" --quiet
        fi

        echo ""
        info "Everything deleted. Nothing billable remains."
        ;;

    *)
        echo "Usage: $0 [start|stop|destroy]"
        exit 1
        ;;
esac
