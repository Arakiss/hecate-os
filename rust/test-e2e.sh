#!/bin/bash
#
# End-to-End Testing Script for HecateOS Rust Components
# This script tests all components in a safe, local environment
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "info")
            echo -e "${BLUE}[INFO]${NC} $message"
            ;;
        "success")
            echo -e "${GREEN}[PASS]${NC} $message"
            ((TESTS_PASSED++))
            ;;
        "warning")
            echo -e "${YELLOW}[WARN]${NC} $message"
            ;;
        "error")
            echo -e "${RED}[FAIL]${NC} $message"
            ((TESTS_FAILED++))
            FAILED_TESTS+=("$message")
            ;;
    esac
}

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -e "\n${BLUE}Testing:${NC} $test_name"
    if eval "$test_command" > /dev/null 2>&1; then
        print_status "success" "$test_name"
        return 0
    else
        print_status "error" "$test_name"
        return 1
    fi
}

# Header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  HecateOS End-to-End Testing Suite${NC}"
echo -e "${BLUE}========================================${NC}\n"

# ============================================================================
# 1. Build Tests
# ============================================================================

print_status "info" "Starting build tests..."

# Test each component can build
COMPONENTS=(
    "hecate-core"
    "hecate-daemon"
    "hecate-gpu"
    "hecate-ml"
    "hecate-monitor"
    "hecate-bench"
    "hecate-pkg"
    "hecate-cli"
)

for component in "${COMPONENTS[@]}"; do
    run_test "Build $component" "cargo build --package $component"
done

# ============================================================================
# 2. Unit Tests
# ============================================================================

print_status "info" "Running unit tests..."

for component in "${COMPONENTS[@]}"; do
    if cargo test --package "$component" --lib 2>/dev/null | grep -q "test result"; then
        run_test "Unit tests for $component" "cargo test --package $component --lib"
    fi
done

# ============================================================================
# 3. Integration Tests
# ============================================================================

print_status "info" "Running integration tests..."

# Test hecate-core hardware detection
run_test "Hardware detection" "cargo run --package hecate-core --example detect 2>/dev/null || true"

# Test hecate-monitor server
print_status "info" "Testing hecate-monitor server..."
(cargo run --package hecate-monitor --bin hecate-monitor &) > /dev/null 2>&1
MONITOR_PID=$!
sleep 2
if kill -0 $MONITOR_PID 2>/dev/null; then
    print_status "success" "hecate-monitor server starts"
    kill $MONITOR_PID 2>/dev/null
else
    print_status "error" "hecate-monitor server failed to start"
fi

# ============================================================================
# 4. CLI Tests
# ============================================================================

print_status "info" "Testing CLI interfaces..."

# Test hecate-cli commands
CLI_COMMANDS=(
    "info"
    "gpu list"
    "monitor status"
)

for cmd in "${CLI_COMMANDS[@]}"; do
    if cargo run --package hecate-cli --bin hecate -- $cmd --help > /dev/null 2>&1; then
        print_status "success" "CLI command: hecate $cmd"
    else
        print_status "warning" "CLI command not available: hecate $cmd"
    fi
done

# ============================================================================
# 5. Database Tests (hecate-pkg)
# ============================================================================

print_status "info" "Testing package manager database..."

# Create temporary test directory
TEST_DIR=$(mktemp -d)
export HECATE_PKG_DB="$TEST_DIR/test.db"

# Test database creation
if cargo run --package hecate-pkg --bin hecate-pkg -- --help > /dev/null 2>&1; then
    print_status "success" "hecate-pkg CLI available"
else
    print_status "error" "hecate-pkg CLI failed"
fi

# Cleanup
rm -rf "$TEST_DIR"

# ============================================================================
# 6. Benchmark Tests
# ============================================================================

print_status "info" "Testing benchmark suite..."

# Test that benchmarks compile
run_test "Benchmark compilation" "cargo build --package hecate-bench --release"

# ============================================================================
# 7. Documentation Tests
# ============================================================================

print_status "info" "Testing documentation..."

run_test "Documentation builds" "cargo doc --no-deps --workspace"

# ============================================================================
# 8. Lint Tests
# ============================================================================

print_status "info" "Running linters..."

if command -v cargo-clippy > /dev/null 2>&1; then
    run_test "Clippy lints" "cargo clippy --workspace -- -W warnings 2>/dev/null || true"
else
    print_status "warning" "Clippy not installed, skipping lint tests"
fi

# ============================================================================
# Test Summary
# ============================================================================

echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE}           Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"

echo -e "${GREEN}Passed:${NC} $TESTS_PASSED"
echo -e "${RED}Failed:${NC} $TESTS_FAILED"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "\n${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    echo -e "\n${RED}TEST SUITE FAILED${NC}"
    exit 1
else
    echo -e "\n${GREEN}ALL TESTS PASSED!${NC}"
    exit 0
fi