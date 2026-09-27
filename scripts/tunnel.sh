#!/usr/bin/env bash
# The app from this machine: https://localhost:3000 while this runs (Ctrl-C
# closes it). Extra ssh arguments are passed on, e.g. -L 8443:localhost:8443
# to reach wheel through the same tunnel.
set -euo pipefail
source "$(dirname "$0")/common.sh"
resolve_project

info "Tunnel open: https://localhost:3000 (Ctrl-C to close)"
vm_ssh -N -L 3000:localhost:3000 "$@"
