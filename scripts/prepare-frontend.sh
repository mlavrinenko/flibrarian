#!/usr/bin/env bash
# Install and build the frontend: flibrarian-web embeds frontend/dist at compile
# time, and the lint recipes need frontend/node_modules. Silent on success.
set -eo pipefail
cd "$(dirname "$0")/../frontend"
log=$(mktemp)
trap 'rm -f "$log"' EXIT
if ! { bun install --frozen-lockfile && bun run build; } >"$log" 2>&1; then
    cat "$log"
    exit 1
fi
