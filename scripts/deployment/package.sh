#!/usr/bin/env bash
# Package release artifacts into compressed archives.

set -euo pipefail

cd "$(dirname "$0")/../.."

dist_dir="dist"
rm -rf "$dist_dir"
mkdir -p "$dist_dir"

for target_dir in target/release target/*/release; do
  [[ -d "$target_dir" ]] || continue
  bin_path="$target_dir/count_lines"
  [[ -x "$bin_path" ]] || continue
  archive_name="$dist_dir/$(basename "${target_dir}").tar.gz"
  tar -czf "$archive_name" -C "$target_dir" count_lines
  echo "Packaged $archive_name"
done
