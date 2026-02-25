#!/usr/bin/env bash
# Run lint-focused checks for count_lines

set -euo pipefail

cd "$(dirname "$0")/../.."

echo "[lint] cargo fmt -- --check"
cargo fmt -- --check

echo "[lint] cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings
