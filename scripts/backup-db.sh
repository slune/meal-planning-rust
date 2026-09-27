#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/common.sh"
resolve_project

BACKUP_DIR="$PROJECT_ROOT/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="meal_planning_${TIMESTAMP}.db"

mkdir -p "$BACKUP_DIR"

echo "Backing up database from $VM_NAME..."

# The DB is at ~/meal-planning/data/ on the VM.
vm_scp "$VM_NAME":~/meal-planning/data/meal_planning.db "$BACKUP_DIR/$BACKUP_FILE"

echo "Saved: backups/$BACKUP_FILE"

# Keep only the 10 most recent backups.
mapfile -t old < <(ls -t "$BACKUP_DIR"/meal_planning_*.db 2>/dev/null | tail -n +11)
if [[ ${#old[@]} -gt 0 ]]; then
    rm -- "${old[@]}"
    echo "Pruned ${#old[@]} old backup(s) (keeping last 10)."
fi
