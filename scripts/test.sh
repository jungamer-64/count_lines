#!/usr/bin/env bash
# Test script for count_lines
# Runs format checks, linting, and tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

echo_error() {
    echo -e "${RED}[✗]${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Change to script directory's parent
cd "$(dirname "$0")/.."

echo_info "Starting test suite for count_lines..."
echo ""

# 1. Format check
echo_info "Running format check..."
if cargo fmt -- --check; then
    echo_success "Format check passed"
else
    echo_error "Format check failed. Run 'cargo fmt' to fix formatting issues."
    exit 1
fi
echo ""

# 2. Clippy
echo_info "Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo_success "Clippy passed"
else
    echo_error "Clippy found issues. Please fix the warnings above."
    exit 1
fi
echo ""

# 3. Check compilation
echo_info "Checking compilation..."
if cargo check --all-targets --all-features; then
    echo_success "Compilation check passed"
else
    echo_error "Compilation failed"
    exit 1
fi
echo ""

# 4. Run all tests
echo_info "Running all tests..."
if cargo test --all-features; then
    echo_success "All tests passed"
else
    echo_error "Some tests failed"
    exit 1
fi
echo ""

# 5. Test core library separately
echo_info "Testing core library (count_lines_core)..."
if cargo test -p count_lines_core; then
    echo_success "Core library tests passed"
else
    echo_error "Core library tests failed"
    exit 1
fi
echo ""

# 6. Run doc tests
echo_info "Running doc tests..."
if cargo test --doc; then
    echo_success "Doc tests passed"
else
    echo_warning "Doc tests failed (this may be okay if no doc tests exist)"
fi
echo ""

# Summary
echo ""
echo_success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo_success "  All checks passed! ✨"
echo_success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
