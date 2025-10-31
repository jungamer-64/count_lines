#!/usr/bin/env bash
# Continuously watch the workspace and rerun tests on change.

set -euo pipefail

cd "$(dirname "$0")/../.."

if command -v cargo-watch >/dev/null 2>&1; then
  cargo watch -x "test --all"
else
  printf 'cargo-watch is not installed. Install it with:\n  cargo install cargo-watch\n' >&2
  exit 1
fi
