#!/usr/bin/env bash
# Benchmark script for count_lines
# Measures performance across various scenarios

set -e

# shellcheck disable=SC2034
# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

echo_benchmark() {
    echo -e "${CYAN}[BENCHMARK]${NC} $1"
}

echo_result() {
    echo -e "${YELLOW}[RESULT]${NC} $1"
}

# Change to repository root
cd "$(dirname "$0")/../.."

echo_info "Building optimized release binary..."
cargo build --release --quiet
BINARY="./target/release/count_lines"

if [ ! -f "$BINARY" ]; then
    echo_error "Binary not found at $BINARY"
    exit 1
fi

echo_success "Binary built successfully"
echo ""

# Create benchmark results directory
RESULTS_DIR="benchmark_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"
echo_info "Results will be saved to: $RESULTS_DIR"
echo ""

# Function to run benchmark and measure time
run_benchmark() {
    local name="$1"
    shift
    # capture remaining args as an array to avoid word-splitting issues
    local -a cmd=("$@")

    echo_benchmark "Running: $name"
    echo "Command: ${cmd[*]}"

    # Run with time measurement
    local start
    start=$(date +%s.%N)
    # execute the command array
    "${cmd[@]}" > /dev/null 2>&1
    local end
    end=$(date +%s.%N)

    local duration
    duration=$(echo "$end - $start" | bc)
    echo_result "Duration: ${duration}s"
    echo "$name,$duration" >> "$RESULTS_DIR/results.csv"
    echo ""
}

# Initialize CSV results file
echo "Benchmark,Duration (seconds)" > "$RESULTS_DIR/results.csv"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Performance Benchmarks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Benchmark 1: Current directory (small)
run_benchmark "Small directory scan" \
    "$BINARY" "."

# Benchmark 2: Recursive scan with limits
run_benchmark "Top 20 files" \
    "$BINARY --top 20 ."

# Benchmark 3: JSON output
run_benchmark "JSON format output" \
    "$BINARY" "--top" "20" "."

# Benchmark 4: YAML output
run_benchmark "YAML format output" \
    "$BINARY" "--format" "json" "."

# Benchmark 5: CSV output
run_benchmark "CSV format output" \
    "$BINARY" "--format" "yaml" "."

# Benchmark 6: Markdown output
run_benchmark "Markdown format output" \
    "$BINARY" "--format" "csv" "."

# Benchmark 7: With sorting
run_benchmark "Sorted by lines (desc)" \
    "$BINARY" "--format" "md" "."

# Benchmark 8: With grouping by extension
run_benchmark "Group by extension" \
    "$BINARY" "--sort" "lines:desc" "."

# Benchmark 9: With filtering
run_benchmark "Filter Rust files only" \
    "$BINARY" "--by" "ext" "."

# Benchmark 10: Git mode (if in git repo)
if [ -d ".git" ]; then
    "$BINARY" "--ext" "rs" "."
        "$BINARY --git ."
fi

# Note: run_benchmark redirects stdout/stderr to /dev/null internally; no explicit shell redirection needed
run_benchmark "With progress indicator" \
    "$BINARY" "--progress" "."
run_benchmark "With progress indicator" \
    "$BINARY --progress . 2>&1"

    "$BINARY" "--ext" "rs,toml,md" "--min-lines" "10" "--sort" "lines:desc" "."
run_benchmark "Complex query (multi-filter)" \
    "$BINARY --ext rs,toml,md --min-lines 10 --sort lines:desc ."

    "$BINARY" "--by" "dir=2" "."
run_benchmark "Group by directory (depth 2)" \
    "$BINARY --by dir=2 ."

# If crates directory exists, benchmark on larger tree
        "$BINARY" "crates/"
    run_benchmark "Deep directory tree (crates/)" \
        "$BINARY crates/"
fi

    "$BINARY" "--format" "jsonl" "--ext" "rs" "--top" "5" "."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Summary Report"
    "$BINARY" "--hidden" "--include" ".*" "--top" "10" "."
echo ""

# Calculate statistics
TOTAL_TIME=$(awk -F',' 'NR>1 {sum+=$2} END {print sum}' "$RESULTS_DIR/results.csv")
AVG_TIME=$(awk -F',' 'NR>1 {sum+=$2; count++} END {print sum/count}' "$RESULTS_DIR/results.csv")
MIN_TIME=$(awk -F',' 'NR>1 {if(min==""){min=$2} if($2<min){min=$2}} END {print min}' "$RESULTS_DIR/results.csv")
MAX_TIME=$(awk -F',' 'NR>1 {if(max==""){max=$2} if($2>max){max=$2}} END {print max}' "$RESULTS_DIR/results.csv")

echo_result "Total time: ${TOTAL_TIME}s"
echo_result "Average time: ${AVG_TIME}s"
echo_result "Fastest: ${MIN_TIME}s"
echo_result "Slowest: ${MAX_TIME}s"
echo ""

# Display top 5 slowest benchmarks
echo "Top 5 slowest operations:"
tail -n +2 "$RESULTS_DIR/results.csv" | sort -t',' -k2 -rn | head -5 | \
    awk -F',' '{printf "  • %-40s %8.4fs\n", $1, $2}'
echo ""

# Display top 5 fastest benchmarks
echo "Top 5 fastest operations:"
tail -n +2 "$RESULTS_DIR/results.csv" | sort -t',' -k2 -n | head -5 | \
    awk -F',' '{printf "  • %-40s %8.4fs\n", $1, $2}'
echo ""

echo_success "Benchmark complete!"
echo_info "Detailed results saved to: $RESULTS_DIR/results.csv"
echo ""

# Create a simple visualization if gnuplot is available
if command -v gnuplot &> /dev/null; then
    echo_info "Generating chart with gnuplot..."
    cat > "$RESULTS_DIR/plot.gnu" << 'EOF'
set terminal png size 1200,800
set output 'benchmark_chart.png'
set title "count_lines Performance Benchmarks"
set ylabel "Duration (seconds)"
set xlabel "Benchmark"
set style data histogram
set style fill solid border -1
set boxwidth 0.8
set xtics rotate by -45
set grid y
set datafile separator ","
plot 'results.csv' using 2:xtic(1) notitle with boxes lc rgb "#4CAF50"
EOF

    cd "$RESULTS_DIR"
    gnuplot plot.gnu
    cd ..

    echo_success "Chart generated: $RESULTS_DIR/benchmark_chart.png"
else
    echo_info "Install gnuplot to generate visualization charts"
fi

echo ""
echo_success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo_success "  Benchmarking complete! ✨"
echo_success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
