#!/usr/bin/env bash
# Build release binaries for common targets.

set -euo pipefail

cd "$(dirname "$0")/../.."

targets=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" "x86_64-apple-darwin")

for target in "${targets[@]}"; do
  echo "[build] cargo build --release --target $target"
  cargo build --release --target "$target"
done
