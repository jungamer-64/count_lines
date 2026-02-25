#!/usr/bin/env bash
# count_lines - Advanced Filtering Examples
#
# This script demonstrates advanced filtering and analysis capabilities.
# These examples show how to perform complex queries and data analysis.

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  count_lines - Advanced Filtering Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Example 1: Multiple extension filtering
echo "Example 1: Count only source code files (Rust, TOML, Markdown)"
echo "Command: count_lines --ext rs,toml,md --top 15"
echo "---"
count_lines --ext rs,toml,md --top 15
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 2: Exclude patterns with glob
echo "Example 2: Exclude test files and dependencies"
echo "Command: count_lines --exclude '*/tests/*' --exclude '*/target/*' --top 10"
echo "---"
count_lines --exclude '*/tests/*' --exclude '*/target/*' --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 3: Include only specific patterns
echo "Example 3: Include only files in src/ and crates/ directories"
echo "Command: count_lines --include 'src/**/*' --include 'crates/**/*' --top 10"
echo "---"
count_lines --include 'src/**/*' --include 'crates/**/*' --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 4: Filter by file size range
echo "Example 4: Show files between 1KB and 50KB"
echo "Command: count_lines --min-size 1024 --max-size 51200 --top 10"
echo "---"
count_lines --min-size 1024 --max-size 51200 --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 5: Filter by line count
echo "Example 5: Show files with more than 100 lines"
echo "Command: count_lines --min-lines 100 --sort lines:desc --top 10"
echo "---"
count_lines --min-lines 100 --sort lines:desc --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 6: Filter by character count range
echo "Example 6: Show files with 1000-10000 characters"
echo "Command: count_lines --min-chars 1000 --max-chars 10000 --top 10"
echo "---"
count_lines --min-chars 1000 --max-chars 10000 --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 7: Expression-based filtering (Rust files with >50 lines)
echo "Example 7: Complex expression filter (Rust files with >50 lines)"
echo "Command: count_lines --filter \"ext == 'rs' && lines > 50\" --top 10"
echo "---"
count_lines --filter "ext == 'rs' && lines > 50" --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 8: Expression with OR conditions
echo "Example 8: Filter Rust OR TOML files with >20 lines"
echo "Command: count_lines --filter \"(ext == 'rs' || ext == 'toml') && lines > 20\""
echo "---"
count_lines --filter "(ext == 'rs' || ext == 'toml') && lines > 20"
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 9: Filter by modification time (files modified in last 7 days)
echo "Example 9: Show files modified in the last 7 days"
echo "Command: count_lines --mtime-within 7d --top 20"
echo "---"
count_lines --mtime-within 7d --top 20
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 10: Group by directory depth
echo "Example 10: Group statistics by directory (depth 2)"
echo "Command: count_lines --by dir=2"
echo "---"
count_lines --by dir=2
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 11: Group by modification time (monthly buckets)
echo "Example 11: Group by modification time (monthly buckets)"
echo "Command: count_lines --by mtime:month"
echo "---"
count_lines --by mtime:month
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 12: Multi-field sorting
echo "Example 12: Sort by extension, then by line count (descending)"
echo "Command: count_lines --sort ext,lines:desc --top 20"
echo "---"
count_lines --sort ext,lines:desc --top 20
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 13: Complex analysis - large Rust files with word count
echo "Example 13: Analyze large Rust files with word count"
echo "Command: count_lines --ext rs --min-lines 200 --words --sort lines:desc"
echo "---"
count_lines --ext rs --min-lines 200 --words --sort lines:desc
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 14: Generate snapshot for comparison
echo "Example 14: Generate JSON snapshot for later comparison"
SNAPSHOT_FILE="snapshot_$(date +%Y%m%d_%H%M%S).json"
echo "Command: count_lines --format json --output $SNAPSHOT_FILE"
echo "---"
count_lines --format json --output "$SNAPSHOT_FILE"
echo "Snapshot saved to: $SNAPSHOT_FILE"
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 15: Combined filters with ratio display
echo "Example 15: Combined filters with percentage display"
echo "Command: count_lines --ext rs,toml --min-lines 10 --ratio --sort lines:desc --top 15"
echo "---"
count_lines --ext rs,toml --min-lines 10 --ratio --sort lines:desc --top 15
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 16: JSONL output for streaming processing
echo "Example 16: JSONL output for streaming/logging"
echo "Command: count_lines --format jsonl --ext rs --top 5"
echo "---"
count_lines --format jsonl --ext rs --top 5
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 17: Complex expression with size and lines
echo "Example 17: Files where size/lines ratio > 100 (average line length)"
echo "Command: count_lines --filter \"lines > 0 && (size / lines) > 100\" --top 10"
echo "---"
count_lines --filter "lines > 0 && (size / lines) > 100" --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 18: Hidden files inclusion
echo "Example 18: Include hidden files in the scan"
echo "Command: count_lines --hidden --include '.*' --top 10"
echo "---"
count_lines --hidden --include '.*' --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 19: Trim root path for cleaner output
if [ -n "$PWD" ]; then
    echo "Example 19: Trim root path for cleaner display"
    echo "Command: count_lines --trim-root \"$PWD\" --top 10"
    echo "---"
    count_lines --trim-root "$PWD" --top 10
    echo ""
    read -r -p "Press Enter to continue..."
    echo ""
fi

# Example 20: Progress indicator for large scans
echo "Example 20: Show progress during scanning"
echo "Command: count_lines --progress --ext rs,toml,md"
echo "---"
count_lines --progress --ext rs,toml,md
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Advanced Examples Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Pro Tips:"
echo "  • Combine multiple filters for precise queries"
echo "  • Use --filter for complex boolean expressions"
echo "  • Group by extension or directory to analyze code structure"
echo "  • Generate JSON snapshots to track changes over time"
echo "  • Use --compare to diff two snapshots"
echo ""
echo "For the complete reference, see: docs/USAGE.md"
echo "For help: count_lines --help --verbose"
echo ""

# Cleanup example snapshot if created
if [ -f "$SNAPSHOT_FILE" ]; then
    echo "Note: Example snapshot '$SNAPSHOT_FILE' was created."
    echo "      You can delete it or use it for testing --compare"
    echo ""
fi
