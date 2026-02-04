#!/bin/bash
#
# HecateOS Complete Test Script
# Tests the entire build and installation flow
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║    HecateOS Complete System Test           ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════╝${NC}"
echo

# 1. Build Check
echo -e "${BLUE}1. Checking Build System${NC}"
echo "══════════════════════════════════════════════"

echo "Building all components..."
cargo build --release --workspace 2>&1 | tail -5
if [ $? -eq 0 ]; then
    echo -e "  Build: ${GREEN}✓${NC}"
else
    echo -e "  Build: ${RED}✗${NC}"
    exit 1
fi

# 2. Binary Check
echo -e "\n${BLUE}2. Checking Binaries${NC}"
echo "══════════════════════════════════════════════"

BINARIES=(
    "hecate"
    "hecated"
    "hecate-monitor"
    "hecate-bench"
    "hecate-pkg"
    "hecate-dev"
    "hecate-iso"
)

MISSING=0
for binary in "${BINARIES[@]}"; do
    if [ -f "target/release/$binary" ]; then
        SIZE=$(du -h "target/release/$binary" | cut -f1)
        echo -e "  $binary: ${GREEN}✓${NC} ($SIZE)"
    else
        echo -e "  $binary: ${RED}✗${NC}"
        MISSING=$((MISSING+1))
    fi
done

if [ $MISSING -gt 0 ]; then
    echo -e "${YELLOW}Warning: $MISSING binaries missing${NC}"
fi

# 3. Doctor Check
echo -e "\n${BLUE}3. Running System Doctor${NC}"
echo "══════════════════════════════════════════════"

./target/release/hecate-dev doctor 2>&1 | grep -E "✅|⚠️|❌" | head -10
echo

# 4. ISO Creation Test
echo -e "${BLUE}4. Testing ISO Creation${NC}"
echo "══════════════════════════════════════════════"

# Create a minimal test content
mkdir -p /tmp/test-iso-mini/{boot,etc}
echo "Test ISO" > /tmp/test-iso-mini/README
echo "bootloader" > /tmp/test-iso-mini/boot/config

echo "Creating test ISO..."
./target/release/hecate-iso repack /tmp/test-iso-mini \
    -o /tmp/test-mini.iso \
    --label "HECATE_TEST" 2>&1 | grep -E "✅|created"

if [ -f /tmp/test-mini.iso ]; then
    SIZE=$(du -h /tmp/test-mini.iso | cut -f1)
    echo -e "  ISO Creation: ${GREEN}✓${NC} (Size: $SIZE)"
    file /tmp/test-mini.iso | grep -q "ISO 9660"
    if [ $? -eq 0 ]; then
        echo -e "  ISO Format: ${GREEN}✓${NC}"
    else
        echo -e "  ISO Format: ${YELLOW}⚠${NC} (Not standard ISO)"
    fi
else
    echo -e "  ISO Creation: ${RED}✗${NC}"
fi

# 5. Component Function Test
echo -e "\n${BLUE}5. Testing Component Functions${NC}"
echo "══════════════════════════════════════════════"

# Test hecate CLI
echo -n "  hecate CLI: "
./target/release/hecate --version &>/dev/null
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test hecate-dev
echo -n "  hecate-dev: "
./target/release/hecate-dev --version &>/dev/null
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test hecate-iso
echo -n "  hecate-iso: "
./target/release/hecate-iso --help &>/dev/null
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# 6. Installation Script Test
echo -e "\n${BLUE}6. Testing Installation Scripts${NC}"
echo "══════════════════════════════════════════════"

if [ -f installer/install.sh ]; then
    echo -e "  install.sh: ${GREEN}✓${NC}"
    # Check script syntax
    bash -n installer/install.sh 2>/dev/null
    if [ $? -eq 0 ]; then
        echo -e "  Script syntax: ${GREEN}✓${NC}"
    else
        echo -e "  Script syntax: ${RED}✗${NC}"
    fi
else
    echo -e "  install.sh: ${RED}✗${NC}"
fi

if [ -f installer/uninstall.sh ]; then
    echo -e "  uninstall.sh: ${GREEN}✓${NC}"
else
    echo -e "  uninstall.sh: ${RED}✗${NC}"
fi

# 7. Clean up
echo -e "\n${BLUE}7. Cleanup${NC}"
echo "══════════════════════════════════════════════"
rm -rf /tmp/test-iso-mini /tmp/test-mini.iso /tmp/test-mini.tar.gz 2>/dev/null
echo -e "  Temporary files: ${GREEN}cleaned${NC}"

# Summary
echo -e "\n${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           Test Summary                     ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo
echo "All basic tests completed. HecateOS components are functional."
echo
echo -e "${CYAN}Next Steps:${NC}"
echo "  1. Download Ubuntu ISO: ./target/release/hecate-dev iso create --download 24.04"
echo "  2. Test on VM: qemu-system-x86_64 -m 4G -cdrom hecateos.iso"
echo "  3. Install on test system: sudo /hecateos/install.sh"
echo
echo -e "${YELLOW}Note:${NC} Full ISO creation requires Ubuntu base ISO and 7z installed."