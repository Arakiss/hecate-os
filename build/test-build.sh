#!/bin/bash
#
# HecateOS Build Test Script
# Tests the build process without actually building the full ISO
#

set -e

SCRIPT_DIR="$(dirname "$0")"
WORK_DIR="$SCRIPT_DIR/iso-build-test"
RUST_DIR="$(dirname "$SCRIPT_DIR")/rust"

echo "==========================================="
echo "  HecateOS Build Test"
echo "==========================================="

# Check disk space
AVAILABLE_SPACE=$(df -BG . | tail -1 | awk '{print $4}' | sed 's/G//')
echo "Available disk space: ${AVAILABLE_SPACE}GB"
if [ "$AVAILABLE_SPACE" -lt 5 ]; then
    echo "Warning: Less than 5GB available, build might fail"
fi

# Check for required tools
echo "Checking required tools..."
for tool in debootstrap squashfs-tools xorriso isolinux; do
    if command -v $tool >/dev/null 2>&1; then
        echo "  ✓ $tool found"
    else
        echo "  ✗ $tool NOT found - would need to install"
    fi
done

# Check Rust binaries
echo ""
echo "Checking HecateOS binaries..."
if [ -d "$RUST_DIR/target/release" ]; then
    for binary in hecated hecate-monitor hecate-bench hecate-pkg \
                  hecate-dev hecate-lint hecate-sign; do
        if [ -f "$RUST_DIR/target/release/$binary" ]; then
            SIZE=$(du -h "$RUST_DIR/target/release/$binary" | cut -f1)
            echo "  ✓ $binary ($SIZE)"
        else
            echo "  ✗ $binary NOT built"
        fi
    done
else
    echo "  ✗ Release binaries not built yet"
    echo "  Run: cd $RUST_DIR && cargo build --release"
fi

# Test creating directory structure
echo ""
echo "Testing directory structure creation..."
mkdir -p "$WORK_DIR"/{iso,squashfs,cd}/{casper,install,isolinux,.disk}
echo "  ✓ Directory structure created"

# Test creating sample files
echo ""
echo "Creating sample configuration files..."

# Test diskdefines
cat > "$WORK_DIR/cd/.disk/info" << EOF
HecateOS Test Build - $(date +%Y%m%d)
EOF
echo "  ✓ Created .disk/info"

# Test ISOLINUX config
cat > "$WORK_DIR/cd/isolinux/isolinux.cfg" << 'EOF'
DEFAULT test
LABEL test
  menu label Test Entry
  kernel /casper/vmlinuz
  append test=true
EOF
echo "  ✓ Created isolinux.cfg"

# Show what would be done
echo ""
echo "Build process would:"
echo "  1. Create Ubuntu base with debootstrap (requires sudo)"
echo "  2. Install packages (~2GB download)"
echo "  3. Copy HecateOS binaries"
echo "  4. Apply optimizations"
echo "  5. Create squashfs (~1-2GB)"
echo "  6. Build ISO image (~2-3GB)"
echo ""
echo "Total estimated: ~10GB temporary space, ~3GB final ISO"

# Cleanup test
rm -rf "$WORK_DIR"
echo ""
echo "Test complete! To build actual ISO:"
echo "  sudo ./build-iso.sh"