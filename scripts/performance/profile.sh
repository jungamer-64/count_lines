#!/usr/bin/env bash
# Collect simple performance profiles for count_lines.

set -euo pipefail

cd "$(dirname "$0")/../.."

if command -v perf >/dev/null 2>&1; then
  perf record --call-graph=dwarf -- cargo run --release -- "$@"
  echo "Profile recorded with perf.data"
else
  echo "perf not found; falling back to `cargo bench`" >&2
  cargo bench
fi
