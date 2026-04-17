#!/usr/bin/env bash
# count_lines - Advanced Filtering Examples

set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  count_lines - Advanced Filtering Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

echo "Example 1: Restrict extensions and exclude build outputs"
echo "Command: count_lines --ext rs,toml,md --exclude 'target/**' --exclude 'node_modules/**' ."
echo "---"
count_lines --ext rs,toml,md --exclude 'target/**' --exclude 'node_modules/**' .
echo

echo "Example 2: Include only selected roots"
echo "Command: count_lines --include 'crates/**' --include 'docs/**' ."
echo "---"
count_lines --include 'crates/**' --include 'docs/**' .
echo

echo "Example 3: Size range filtering"
echo "Command: count_lines --min-size 1KiB --max-size 256KiB ."
echo "---"
count_lines --min-size 1KiB --max-size 256KiB .
echo

echo "Example 4: Lines and chars thresholds"
echo "Command: count_lines --min-lines 20 --max-lines 5000 --min-chars 200 ."
echo "---"
count_lines --min-lines 20 --max-lines 5000 --min-chars 200 .
echo

echo "Example 5: Word-aware filtering"
echo "Command: count_lines --words --min-words 50 --sort words:desc ."
echo "---"
count_lines --words --min-words 50 --sort words:desc .
echo

echo "Example 6: Mtime window filtering"
echo "Command: count_lines --mtime-since 2024-01-01 --mtime-until 2030-01-01 ."
echo "---"
count_lines --mtime-since 2024-01-01 --mtime-until 2030-01-01 .
echo

echo "Example 7: Extension language remap"
echo "Command: count_lines --map-ext h=cpp --map-ext inc=cpp --sloc ."
echo "---"
count_lines --map-ext h=cpp --map-ext inc=cpp --sloc .
echo

echo "Example 8: Deeper walk tuning"
echo "Command: count_lines --max-depth 4 --jobs 4 --walk-threads 4 ."
echo "---"
count_lines --max-depth 4 --jobs 4 --walk-threads 4 .
echo

echo "Example 9: Override patterns"
echo "Command: count_lines --override-exclude 'target/**' --override-include 'crates/**' ."
echo "---"
count_lines --override-exclude 'target/**' --override-include 'crates/**' .
echo

echo "Example 10: JSONL stream output"
echo "Command: count_lines --format jsonl --ext rs ."
echo "---"
count_lines --format jsonl --ext rs .
echo

echo "Example 11: Snapshot compare workflow"
OLD="snapshot_old.json"
NEW="snapshot_new.json"
echo "Command: count_lines --format json . > $OLD"
count_lines --format json . > "$OLD"
echo "Command: count_lines --format json . > $NEW"
count_lines --format json . > "$NEW"
echo "Command: count_lines --compare $OLD $NEW"
count_lines --compare "$OLD" "$NEW"
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Advanced Examples Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "For full option details: docs/user/CLI_REFERENCE.md"
echo "For help: count_lines --help"
echo
