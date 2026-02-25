#!/usr/bin/env bash
# CI helper for release automation.

set -euo pipefail

cd "$(dirname "$0")/../.."

cargo fmt
cargo test --all --all-features
cargo build --release
