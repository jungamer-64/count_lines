#!/usr/bin/env bash
# CI validation entry point.

set -euo pipefail

cd "$(dirname "$0")/../.."

./scripts/development/test.sh
