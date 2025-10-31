#!/usr/bin/env bash
# count_lines - Basic Usage Examples
#
# This script demonstrates the most common usage patterns for count_lines.
# Run this script from any directory to see various counting options.

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  count_lines - Basic Usage Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Example 1: Count files in current directory
echo "Example 1: Count all files in current directory"
echo "Command: count_lines"
echo "---"
count_lines
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 2: Count with top N limit
echo "Example 2: Show top 10 largest files"
echo "Command: count_lines --top 10"
echo "---"
count_lines --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 3: Count specific file types
echo "Example 3: Count only Rust source files"
echo "Command: count_lines --ext rs"
echo "---"
count_lines --ext rs
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 4: JSON output
echo "Example 4: Output in JSON format"
echo "Command: count_lines --format json --top 5"
echo "---"
count_lines --format json --top 5
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 5: CSV output for spreadsheet import
echo "Example 5: Output in CSV format with total row"
echo "Command: count_lines --format csv --total-row --top 5"
echo "---"
count_lines --format csv --total-row --top 5
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 6: Sort by different columns
echo "Example 6: Sort by line count (descending)"
echo "Command: count_lines --sort lines:desc --top 10"
echo "---"
count_lines --sort lines:desc --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 7: Group by file extension
echo "Example 7: Group statistics by file extension"
echo "Command: count_lines --by ext"
echo "---"
count_lines --by ext
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 8: Filter by size
echo "Example 8: Show files larger than 1KB"
echo "Command: count_lines --min-size 1024 --top 10"
echo "---"
count_lines --min-size 1024 --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 9: Markdown output for documentation
echo "Example 9: Generate Markdown table for documentation"
echo "Command: count_lines --format md --top 10"
echo "---"
count_lines --format md --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 10: Show percentages
echo "Example 10: Display with ratio/percentage columns"
echo "Command: count_lines --ratio --top 10"
echo "---"
count_lines --ratio --top 10
echo ""
read -r -p "Press Enter to continue..."
echo ""

# Example 11: Git mode (respects .gitignore)
if [ -d ".git" ]; then
    echo "Example 11: Scan with Git mode (respects .gitignore)"
    echo "Command: count_lines --git --top 10"
    echo "---"
    count_lines --git --top 10
    echo ""
else
    echo "Example 11: Skipped (not in a Git repository)"
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Examples Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "For more options, run: count_lines --help"
echo "For detailed documentation, see: docs/USAGE.md"
echo ""
