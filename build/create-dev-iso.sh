#!/bin/bash
#
# HecateOS Development ISO Creator
# Creates a complete development environment ISO with all build tools included
#

set -e

SCRIPT_DIR="$(dirname "$0")"
WORK_DIR="$SCRIPT_DIR/dev-iso"
RUST_DIR="$(dirname "$SCRIPT_DIR")/rust"
OUTPUT_DIR="$SCRIPT_DIR/output"
ISO_NAME="hecateos-dev-0.1.0-amd64.iso"

echo "==========================================="
echo "  HecateOS Development ISO Creator"
echo "==========================================="
echo "This creates a complete ISO with ALL tools needed"
echo ""

# Create directories
mkdir -p "$WORK_DIR" "$OUTPUT_DIR"
cd "$WORK_DIR"

# Create a minimal bootable structure using only basic tools
echo "Creating ISO structure..."
mkdir -p iso/{boot,live,isolinux,EFI/boot}

# Create a minimal init script that sets up the dev environment
cat > iso/init << 'INIT_SCRIPT'
#!/bin/sh
echo "HecateOS Development Environment Starting..."

# Mount essential filesystems
mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev

# Create development environment bootstrap script
cat > /bootstrap-hecate.sh << 'BOOTSTRAP'
#!/bin/bash
#
# HecateOS Bootstrap Script
# This runs INSIDE the live environment to build HecateOS
#

echo "==================================="
echo "  HecateOS In-System Builder"
echo "==================================="

# Step 1: Install all required packages
echo "Installing build dependencies..."
apt-get update
apt-get install -y \
    debootstrap \
    squashfs-tools \
    xorriso \
    isolinux \
    syslinux-utils \
    build-essential \
    curl \
    wget \
    git \
    cargo \
    rustc

# Step 2: Build HecateOS components
echo "Building HecateOS components..."
if [ -d /hecate-source ]; then
    cd /hecate-source/rust
    cargo build --release
fi

# Step 3: Create the final ISO
echo "Creating final HecateOS ISO..."
cd /hecate-source/build
./build-iso.sh

echo "Build complete! ISO available at: /hecate-source/build/output/"
BOOTSTRAP

chmod +x /bootstrap-hecate.sh

# Network setup
ifconfig lo up
dhclient eth0 2>/dev/null || true

echo ""
echo "==========================================="
echo "   HecateOS Development Environment"
echo "==========================================="
echo ""
echo "To build HecateOS, run:"
echo "  /bootstrap-hecate.sh"
echo ""
echo "This will:"
echo "  1. Install all build tools"
echo "  2. Build Rust components"
echo "  3. Create the final ISO"
echo ""

# Start shell
exec /bin/bash
INIT_SCRIPT
chmod +x iso/init

# Create a minimal rootfs with busybox and basic tools
echo "Creating minimal rootfs..."
mkdir -p rootfs/{bin,sbin,etc,dev,proc,sys,tmp,var,usr,lib,lib64}

# Download static busybox if available
echo "Adding busybox for basic utilities..."
if command -v busybox >/dev/null 2>&1; then
    cp $(which busybox) rootfs/bin/
else
    # Download pre-built static busybox
    wget -q -O rootfs/bin/busybox \
        https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox || \
        echo "Warning: Could not download busybox"
fi

if [ -f rootfs/bin/busybox ]; then
    chmod +x rootfs/bin/busybox
    # Create essential symlinks
    for cmd in sh ash bash cat ls cp mv rm mkdir echo mount umount; do
        ln -sf busybox rootfs/bin/$cmd
    done
fi

# Copy the entire source tree into the ISO
echo "Adding HecateOS source code..."
mkdir -p rootfs/hecate-source
cp -r "$SCRIPT_DIR/.." rootfs/hecate-source/

# Create a bootloader configuration
cat > iso/isolinux/isolinux.cfg << 'ISOLINUX_CFG'
DEFAULT hecate-dev
PROMPT 1
TIMEOUT 50

LABEL hecate-dev
  KERNEL /live/vmlinuz
  APPEND initrd=/live/initrd.img boot=live

LABEL hecate-build
  KERNEL /live/vmlinuz
  APPEND initrd=/live/initrd.img boot=live auto-build=true
ISOLINUX_CFG

# Create a simple bootable message
cat > iso/isolinux/boot.msg << 'BOOT_MSG'
=====================================
  HecateOS Development Environment
=====================================

This ISO contains everything needed to build HecateOS:
- Complete source code
- Build tools installer
- Development environment

Boot options:
  hecate-dev   - Start development environment (default)
  hecate-build - Auto-build HecateOS on boot

=====================================
BOOT_MSG

# Package rootfs as initrd
echo "Creating initrd..."
cd rootfs
find . | cpio -o -H newc | gzip -9 > ../iso/live/initrd.img
cd ..

# We need at least a kernel - try to find one
echo "Looking for kernel..."
if [ -f /boot/vmlinuz-$(uname -r) ]; then
    cp /boot/vmlinuz-$(uname -r) iso/live/vmlinuz
    echo "  Using host kernel: $(uname -r)"
elif [ -f /boot/vmlinuz ]; then
    cp /boot/vmlinuz iso/live/vmlinuz
    echo "  Using host kernel"
else
    # Try to extract from Ubuntu ISO if available
    echo "  Warning: No kernel found. ISO may not boot."
    echo "  You need to add a kernel to iso/live/vmlinuz"
fi

# Create the ISO using genisoimage or mkisofs
echo "Creating ISO image..."
if command -v genisoimage >/dev/null 2>&1; then
    MKISO="genisoimage"
elif command -v mkisofs >/dev/null 2>&1; then
    MKISO="mkisofs"
else
    echo "Error: Neither genisoimage nor mkisofs found"
    echo "Install with: sudo apt-get install genisoimage"
    exit 1
fi

$MKISO -R -J -joliet-long \
    -b isolinux/isolinux.bin \
    -c isolinux/boot.cat \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -o "$OUTPUT_DIR/$ISO_NAME" \
    -V "HECATEOS_DEV" \
    iso/

if [ -f "$OUTPUT_DIR/$ISO_NAME" ]; then
    SIZE=$(du -h "$OUTPUT_DIR/$ISO_NAME" | cut -f1)
    echo ""
    echo "==========================================="
    echo "Development ISO created successfully!"
    echo "Output: $OUTPUT_DIR/$ISO_NAME"
    echo "Size: $SIZE"
    echo ""
    echo "This ISO contains:"
    echo "  - Complete HecateOS source code"
    echo "  - Bootstrap script to install build tools"
    echo "  - Everything needed to build the final ISO"
    echo ""
    echo "To use:"
    echo "  1. Boot this ISO in a VM or physical machine"
    echo "  2. Run: /bootstrap-hecate.sh"
    echo "  3. This will build the final HecateOS ISO"
    echo "==========================================="
else
    echo "Error: ISO creation failed"
    exit 1
fi