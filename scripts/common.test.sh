#!/usr/bin/env bash
# The GCP helpers and the scripts' names: bash scripts/common.test.sh
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
check() { # name expected actual
    if [[ "$2" == "$3" ]]; then
        echo "ok   $1"
    else
        echo "FAIL $1"
        printf '  expected: %q\n  actual:   %q\n' "$2" "$3"
        fail=1
    fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
printf '#!/bin/sh\necho "$*"\n' > "$tmp/gcloud"
chmod +x "$tmp/gcloud"

source scripts/common.sh
check "vm_ssh goes to homelab-vm through IAP" \
    "compute ssh homelab-vm --zone=us-east1-b --project=p-123 --tunnel-through-iap --quiet -- uptime" \
    "$(PROJECT_ID=p-123 PATH="$tmp:$PATH" vm_ssh uptime)"
check "vm_scp goes through IAP" \
    "compute scp --zone=us-east1-b --project=p-123 --tunnel-through-iap --quiet a homelab-vm:b" \
    "$(PROJECT_ID=p-123 PATH="$tmp:$PATH" vm_scp a homelab-vm:b)"
check "no script uses the old names or switches the active project" \
    "" \
    "$(grep -nE 'meal-planning-vm|meal-planning-ip|meal-planning-allow-app|config set project' scripts/*.sh |
        grep -v '^scripts/common.test.sh' || true)"
check "every ssh/scp goes through vm_ssh/vm_scp" \
    "" \
    "$(grep -nE 'gcloud compute (ssh|scp)' scripts/*.sh | grep -vE '^scripts/common(\.test)?\.sh' || true)"

exit "$fail"
