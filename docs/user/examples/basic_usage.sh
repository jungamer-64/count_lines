#!/usr/bin/env bash
# count_lines - Basic Usage Examples

set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  count_lines - Basic Usage Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

echo "Example 1: Count all files in current directory"
echo "Command: count_lines ."
echo "---"
count_lines .
echo

echo "Example 2: Count only Rust files"
echo "Command: count_lines --ext rs ."
echo "---"
count_lines --ext rs .
echo

echo "Example 3: Sort by lines desc, then chars desc"
echo "Command: count_lines --sort lines:desc,chars:desc ."
echo "---"
count_lines --sort lines:desc,chars:desc .
echo

echo "Example 4: CSV output with TOTAL row"
echo "Command: count_lines --format csv --total-row ."
echo "---"
count_lines --format csv --total-row .
echo

echo "Example 5: JSON snapshot"
echo "Command: count_lines --format json . > snapshot_basic.json"
echo "---"
count_lines --format json . > snapshot_basic.json
echo "Saved: snapshot_basic.json"
echo

echo "Example 6: Include / exclude patterns"
echo "Command: count_lines --include 'crates/**' --exclude 'target/**' ."
echo "---"
count_lines --include 'crates/**' --exclude 'target/**' .
echo

echo "Example 7: Count words"
echo "Command: count_lines --words --sort words:desc ."
echo "---"
count_lines --words --sort words:desc .
echo

echo "Example 8: Count SLOC"
echo "Command: count_lines --sloc --sort sloc:desc ."
echo "---"
count_lines --sloc --sort sloc:desc .
echo

echo "Example 9: Ignore .gitignore rules (scan everything)"
echo "Command: count_lines --no-gitignore ."
echo "---"
count_lines --no-gitignore .
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Examples Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "For more options, run: count_lines --help"
echo "See docs: docs/user/CLI_REFERENCE.md"
echo
