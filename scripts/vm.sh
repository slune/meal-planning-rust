#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-}"
source "$(dirname "$0")/common.sh"
[[ -f "$PROJECT_ROOT/.env" ]] && set -a && source "$PROJECT_ROOT/.env" && set +a
resolve_project

vm_status() {
    gcloud compute instances describe "$VM_NAME" --zone="$ZONE" --project="$PROJECT_ID" \
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
        if [[ "$(wait_for_stable)" == "NOT_FOUND" ]]; then
            warn "VM '$VM_NAME' does not exist in zone '$ZONE'. Create it first."
            exit 1
        fi

        # ── Network: no external IP; NAT out, IAP in ────────────────────────
        ensure_network

        # ── Start VM ────────────────────────────────────────────────────────
        if [[ "$(vm_status)" == "RUNNING" ]]; then
            info "VM is already running."
        else
            info "Starting VM..."
            gcloud compute instances start "$VM_NAME" --zone="$ZONE" --project="$PROJECT_ID"
        fi

        echo ""
        info "Reach the app: scripts/tunnel.sh, then https://localhost:3000"
        ;;

    stop)
        STATUS=$(wait_for_stable)
        if [[ "$STATUS" == "NOT_FOUND" ]]; then
            info "VM does not exist; nothing to stop."
        elif [[ "$STATUS" == "TERMINATED" ]]; then
            info "VM is already stopped."
        else
            info "Stopping VM..."
            gcloud compute instances stop "$VM_NAME" --zone="$ZONE" --project="$PROJECT_ID"
        fi

        echo ""
        warn "wheel runs on this VM too: its scheduled runs are skipped until the next start."
        info "VM stopped. Only the boot disk remains billable."
        ;;

    destroy)
        echo ""
        warn "This will permanently delete the VM, its boot disk (wheel's data too),"
        warn "the Cloud NAT and router, and the IAP SSH firewall rule."
        read -rp "Type 'yes' to confirm: " confirm
        [[ "$confirm" == "yes" ]] || { echo "Aborted."; exit 1; }

        if [[ "$(wait_for_stable)" == "NOT_FOUND" ]]; then
            info "VM already deleted."
        else
            info "Deleting VM and boot disk..."
            gcloud compute instances delete "$VM_NAME" --zone="$ZONE" --project="$PROJECT_ID" \
                --delete-disks=all --quiet
        fi

        if gcloud compute routers describe "$ROUTER" --region="$REGION" --project="$PROJECT_ID" &>/dev/null; then
            info "Deleting Cloud NAT and router..."
            gcloud compute routers nats delete "$NAT" --router="$ROUTER" --region="$REGION" \
                --project="$PROJECT_ID" --quiet || true
            gcloud compute routers delete "$ROUTER" --region="$REGION" --project="$PROJECT_ID" --quiet
        fi

        if gcloud compute firewall-rules describe "$IAP_RULE" --project="$PROJECT_ID" &>/dev/null; then
            info "Deleting firewall rule..."
            gcloud compute firewall-rules delete "$IAP_RULE" --project="$PROJECT_ID" --quiet
        fi

        echo ""
        info "Everything deleted. Nothing billable remains."
        ;;

    *)
        echo "Usage: $0 [start|stop|destroy]"
        exit 1
        ;;
esac
