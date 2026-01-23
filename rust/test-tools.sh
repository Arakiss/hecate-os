#!/bin/bash
# Test script for HecateOS development tools

set -e

echo "🔍 Checking HecateOS Development Tools..."
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Rust is installed
check_rust() {
    if command -v rustc &> /dev/null; then
        echo -e "${GREEN}✓${NC} Rust installed: $(rustc --version)"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} Rust not installed - skipping compilation tests"
        return 1
    fi
}

# Check Cargo.toml files
check_cargo_files() {
    echo -e "\n📦 Checking Cargo.toml files..."
    
    local tools=("hecate-dev" "hecate-lint" "hecate-hooks" "hecate-changelog" "hecate-deps" "hecate-arch")
    
    for tool in "${tools[@]}"; do
        if [ -f "$tool/Cargo.toml" ]; then
            echo -e "${GREEN}✓${NC} $tool/Cargo.toml exists"
            
            # Check for required fields
            if grep -q "name = \"$tool\"" "$tool/Cargo.toml"; then
                echo -e "  ${GREEN}✓${NC} Package name correct"
            else
                echo -e "  ${RED}✗${NC} Package name incorrect"
            fi
        else
            echo -e "${RED}✗${NC} $tool/Cargo.toml missing"
        fi
    done
}

# Check source files
check_source_files() {
    echo -e "\n📝 Checking source files..."
    
    local tools=("hecate-dev" "hecate-lint" "hecate-hooks" "hecate-changelog" "hecate-deps" "hecate-arch")
    
    for tool in "${tools[@]}"; do
        if [ -f "$tool/src/main.rs" ]; then
            echo -e "${GREEN}✓${NC} $tool/src/main.rs exists"
            
            # Check for basic structure
            if grep -q "fn main()" "$tool/src/main.rs"; then
                echo -e "  ${GREEN}✓${NC} Has main function"
            else
                echo -e "  ${RED}✗${NC} Missing main function"
            fi
        else
            echo -e "${RED}✗${NC} $tool/src/main.rs missing"
        fi
    done
}

# Check workspace configuration
check_workspace() {
    echo -e "\n⚙️  Checking workspace configuration..."
    
    if [ -f "Cargo.toml" ]; then
        echo -e "${GREEN}✓${NC} Workspace Cargo.toml exists"
        
        # Check all tools are in workspace
        local tools=("hecate-dev" "hecate-lint" "hecate-hooks" "hecate-changelog" "hecate-deps" "hecate-arch")
        
        for tool in "${tools[@]}"; do
            if grep -q "\"$tool\"" Cargo.toml; then
                echo -e "  ${GREEN}✓${NC} $tool in workspace members"
            else
                echo -e "  ${RED}✗${NC} $tool not in workspace members"
            fi
        done
    else
        echo -e "${RED}✗${NC} Workspace Cargo.toml missing"
    fi
}

# Check for module dependencies
check_modules() {
    echo -e "\n🔗 Checking hecate-dev modules..."
    
    if [ -f "hecate-dev/src/main.rs" ]; then
        local modules=("version" "commit" "check" "release")
        
        for module in "${modules[@]}"; do
            if [ -f "hecate-dev/src/$module.rs" ]; then
                echo -e "${GREEN}✓${NC} Module $module.rs exists"
            else
                echo -e "${RED}✗${NC} Module $module.rs missing"
            fi
        done
    fi
}

# Try to build if Rust is available
try_build() {
    if check_rust; then
        echo -e "\n🔨 Attempting to build tools..."
        
        # Try cargo check first (faster than build)
        if cargo check --workspace 2>/dev/null; then
            echo -e "${GREEN}✓${NC} All tools compile successfully!"
        else
            echo -e "${RED}✗${NC} Compilation errors detected"
            echo "Run 'cargo check --workspace' for details"
        fi
    fi
}

# Main execution
main() {
    echo "Running from: $(pwd)"
    
    check_cargo_files
    check_source_files
    check_workspace
    check_modules
    try_build
    
    echo -e "\n========================================="
    echo "📊 Test Summary"
    echo "========================================="
    
    # Count successes and failures
    local success_count=$(grep -c "✓" /tmp/test-output 2>/dev/null || echo "0")
    local failure_count=$(grep -c "✗" /tmp/test-output 2>/dev/null || echo "0")
    
    if [ "$failure_count" = "0" ]; then
        echo -e "${GREEN}All checks passed!${NC}"
    else
        echo -e "${YELLOW}Some issues found - review above${NC}"
    fi
}

# Run with output capture for summary
main 2>&1 | tee /tmp/test-output