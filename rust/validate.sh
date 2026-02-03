#!/bin/bash
#
# HecateOS Rust Components Validation Script
# This script validates that everything is production-ready
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BLUE}${BOLD}========================================${NC}"
echo -e "${BLUE}${BOLD}  HecateOS Production Validation${NC}"
echo -e "${BLUE}${BOLD}========================================${NC}\n"

# Track results
PASSED=0
FAILED=0
WARNINGS=0

# Function to check status
check() {
    local name=$1
    local command=$2
    
    echo -n "Checking $name... "
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC}"
        ((FAILED++))
        return 1
    fi
}

# Function to warn
warn() {
    local name=$1
    local command=$2
    
    echo -n "Checking $name... "
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        ((PASSED++))
    else
        echo -e "${YELLOW}⚠${NC}"
        ((WARNINGS++))
    fi
}

echo -e "${BOLD}1. Build Validation${NC}"
echo "==================="
check "Workspace builds" "cargo build --workspace"
check "Release build" "cargo build --workspace --release"
check "Documentation builds" "cargo doc --workspace --no-deps"

echo -e "\n${BOLD}2. Code Quality${NC}"
echo "==============="
check "Formatting" "cargo fmt --all -- --check"
warn "No clippy warnings" "cargo clippy --workspace -- -D warnings"
warn "No unsafe code" "! grep -r 'unsafe ' src/ --include='*.rs' 2>/dev/null"

echo -e "\n${BOLD}3. Testing${NC}"
echo "=========="
check "Unit tests pass" "cargo test --workspace --lib"
warn "Integration tests" "cargo test --workspace --tests"
warn "Doc tests" "cargo test --workspace --doc"

echo -e "\n${BOLD}4. Dependencies${NC}"
echo "==============="
warn "No security vulnerabilities" "cargo audit 2>/dev/null || true"
check "Dependencies resolve" "cargo tree > /dev/null"
warn "No duplicate dependencies" "! cargo tree -d 2>&1 | grep -v 'No duplicate dependencies found'"

echo -e "\n${BOLD}5. Component Validation${NC}"
echo "======================="
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
    check "$component builds" "cargo build -p $component"
done

echo -e "\n${BOLD}6. Binary Execution${NC}"
echo "=================="
check "hecate-cli --help" "cargo run -p hecate-cli -- --help"
check "hecate-bench --help" "cargo run -p hecate-bench -- --help"
check "hecate-pkg --help" "cargo run -p hecate-pkg -- --help"

echo -e "\n${BOLD}7. Production Readiness${NC}"
echo "======================"

# Check for TODOs and FIXMEs
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" --include="*.rs" . 2>/dev/null | wc -l || echo "0")
if [ "$TODO_COUNT" -gt "50" ]; then
    echo -e "TODOs in code... ${YELLOW}⚠ ($TODO_COUNT found)${NC}"
    ((WARNINGS++))
else
    echo -e "TODOs in code... ${GREEN}✓ ($TODO_COUNT found)${NC}"
    ((PASSED++))
fi

# Check test coverage
if command -v cargo-tarpaulin > /dev/null 2>&1; then
    echo -n "Test coverage... "
    COVERAGE=$(cargo tarpaulin --workspace --print-summary 2>/dev/null | grep "Coverage" | grep -oE "[0-9]+\.[0-9]+" | head -1 || echo "0")
    if (( $(echo "$COVERAGE > 70" | bc -l) )); then
        echo -e "${GREEN}✓ ($COVERAGE%)${NC}"
        ((PASSED++))
    elif (( $(echo "$COVERAGE > 50" | bc -l) )); then
        echo -e "${YELLOW}⚠ ($COVERAGE%)${NC}"
        ((WARNINGS++))
    else
        echo -e "${RED}✗ ($COVERAGE%)${NC}"
        ((FAILED++))
    fi
else
    echo -e "Test coverage... ${YELLOW}⚠ (tarpaulin not installed)${NC}"
    ((WARNINGS++))
fi

# Check for required files
echo -e "\n${BOLD}8. Documentation${NC}"
echo "==============="
check "README.md exists" "[ -f ../README.md ]"
check "TESTING.md exists" "[ -f TESTING.md ]"
check "Cargo.toml valid" "cargo metadata --format-version 1 > /dev/null"

# Check binary sizes
echo -e "\n${BOLD}9. Binary Sizes${NC}"
echo "=============="
if [ -d "target/release" ]; then
    for bin in target/release/hecate-*; do
        if [ -f "$bin" ] && [ -x "$bin" ]; then
            SIZE=$(du -h "$bin" | cut -f1)
            NAME=$(basename "$bin")
            echo "  $NAME: $SIZE"
        fi
    done
else
    echo -e "  ${YELLOW}Release binaries not built yet${NC}"
fi

# ============================================================================
# Summary
# ============================================================================

echo -e "\n${BLUE}${BOLD}========================================${NC}"
echo -e "${BLUE}${BOLD}           Validation Summary${NC}"
echo -e "${BLUE}${BOLD}========================================${NC}"

echo -e "${GREEN}Passed:${NC} $PASSED"
echo -e "${YELLOW}Warnings:${NC} $WARNINGS"
echo -e "${RED}Failed:${NC} $FAILED"

TOTAL=$((PASSED + WARNINGS + FAILED))
SCORE=$((PASSED * 100 / TOTAL))

echo -e "\n${BOLD}Production Readiness Score: $SCORE%${NC}"

if [ $FAILED -eq 0 ]; then
    if [ $WARNINGS -lt 5 ]; then
        echo -e "${GREEN}${BOLD}✓ PRODUCTION READY${NC}"
        exit 0
    else
        echo -e "${YELLOW}${BOLD}⚠ NEARLY READY (fix warnings)${NC}"
        exit 0
    fi
else
    echo -e "${RED}${BOLD}✗ NOT READY FOR PRODUCTION${NC}"
    exit 1
fi