#!/bin/bash
#
# HecateOS Simple ISO Builder
# Uses Ubuntu Server ISO as base and adds HecateOS components
#

set -e

SCRIPT_DIR="$(dirname "$0")"
WORK_DIR="$SCRIPT_DIR/simple-iso"
RUST_DIR="$(dirname "$SCRIPT_DIR")/rust"
OUTPUT_DIR="$SCRIPT_DIR/output"

# Ubuntu Server ISO (you need to download this)
UBUNTU_ISO="ubuntu-24.04-live-server-amd64.iso"
UBUNTU_URL="https://releases.ubuntu.com/24.04/ubuntu-24.04-live-server-amd64.iso"

echo "==========================================="
echo "  HecateOS Simple ISO Builder"
echo "==========================================="

# Check if Ubuntu ISO exists
if [ ! -f "$UBUNTU_ISO" ]; then
    echo "Ubuntu Server ISO not found."
    echo "Download from: $UBUNTU_URL"
    echo "Or run: wget $UBUNTU_URL"
    exit 1
fi

# Create work directory
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR" "$OUTPUT_DIR"
cd "$WORK_DIR"

echo "Extracting Ubuntu ISO..."
# Extract ISO contents
mkdir -p iso_mount iso_extract
sudo mount -o loop "../$UBUNTU_ISO" iso_mount
cp -r iso_mount/* iso_extract/ 2>/dev/null || true
cp -r iso_mount/.disk iso_extract/ 2>/dev/null || true
sudo umount iso_mount
rmdir iso_mount

echo "Adding HecateOS components..."

# Create HecateOS directory in the ISO
mkdir -p iso_extract/hecateos

# Add compiled binaries if they exist
if [ -d "$RUST_DIR/target/release" ]; then
    echo "Copying HecateOS binaries..."
    mkdir -p iso_extract/hecateos/bin
    for binary in hecated hecate-monitor hecate-bench hecate-pkg \
                  hecate-dev hecate-lint hecate-sign; do
        if [ -f "$RUST_DIR/target/release/$binary" ]; then
            cp "$RUST_DIR/target/release/$binary" iso_extract/hecateos/bin/
            echo "  Added $binary"
        fi
    done
else
    echo "Warning: Rust binaries not built. Adding source code instead..."
    cp -r "$RUST_DIR" iso_extract/hecateos/rust-source
fi

# Add installation script
cat > iso_extract/hecateos/install.sh << 'INSTALL_SCRIPT'
#!/bin/bash
#
# HecateOS Post-Install Script
# Run this after Ubuntu installation to add HecateOS components
#

echo "==========================================="
echo "  HecateOS Component Installer"
echo "==========================================="

# Install HecateOS binaries
if [ -d /cdrom/hecateos/bin ]; then
    echo "Installing HecateOS binaries..."
    sudo cp -r /cdrom/hecateos/bin/* /usr/local/bin/
    sudo chmod +x /usr/local/bin/hecate*
fi

# Build from source if needed
if [ -d /cdrom/hecateos/rust-source ]; then
    echo "Building from source..."
    sudo apt-get update
    sudo apt-get install -y build-essential cargo
    cp -r /cdrom/hecateos/rust-source /tmp/
    cd /tmp/rust-source
    cargo build --release
    sudo cp target/release/hecate* /usr/local/bin/
fi

# Apply optimizations
echo "Applying HecateOS optimizations..."

# Kernel parameters
sudo tee /etc/sysctl.d/99-hecateos.conf > /dev/null << 'EOF'
vm.swappiness = 10
vm.vfs_cache_pressure = 50
net.core.default_qdisc = fq_codel
net.ipv4.tcp_congestion = bbr
EOF

# CPU governor
echo "performance" | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Create systemd services
sudo tee /etc/systemd/system/hecated.service > /dev/null << 'EOF'
[Unit]
Description=HecateOS System Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/hecated
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable hecated
sudo systemctl start hecated

echo "==========================================="
echo "HecateOS components installed!"
echo "Reboot to complete setup."
echo "==========================================="
INSTALL_SCRIPT

chmod +x iso_extract/hecateos/install.sh

# Modify boot menu to add HecateOS option
if [ -f iso_extract/boot/grub/grub.cfg ]; then
    # Backup original
    cp iso_extract/boot/grub/grub.cfg iso_extract/boot/grub/grub.cfg.orig
    
    # Add HecateOS entry (simplified - would need proper modification)
    echo "Modified GRUB config for HecateOS branding"
fi

# Update disk info
echo "HecateOS 0.1.0 based on Ubuntu 24.04 LTS - $(date +%Y%m%d)" > iso_extract/.disk/info

# Create new ISO
echo "Creating new ISO..."
if command -v xorriso >/dev/null 2>&1; then
    xorriso -as mkisofs \
        -r -J -joliet-long \
        -b isolinux/isolinux.bin \
        -c isolinux/boot.cat \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        -eltorito-alt-boot \
        -e boot/grub/efi.img \
        -no-emul-boot \
        -o "$OUTPUT_DIR/hecateos-0.1.0-amd64.iso" \
        -V "HecateOS" \
        iso_extract
elif command -v genisoimage >/dev/null 2>&1; then
    genisoimage -r -J -joliet-long \
        -b isolinux/isolinux.bin \
        -c isolinux/boot.cat \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        -o "$OUTPUT_DIR/hecateos-0.1.0-amd64.iso" \
        -V "HecateOS" \
        iso_extract
else
    echo "Error: No ISO creation tool found (xorriso or genisoimage)"
    echo "Install with: sudo apt-get install xorriso"
    exit 1
fi

if [ -f "$OUTPUT_DIR/hecateos-0.1.0-amd64.iso" ]; then
    SIZE=$(du -h "$OUTPUT_DIR/hecateos-0.1.0-amd64.iso" | cut -f1)
    echo ""
    echo "==========================================="
    echo "ISO created successfully!"
    echo "Output: $OUTPUT_DIR/hecateos-0.1.0-amd64.iso"
    echo "Size: $SIZE"
    echo ""
    echo "Installation process:"
    echo "  1. Boot the ISO"
    echo "  2. Install Ubuntu normally"
    echo "  3. After installation, run:"
    echo "     sudo /cdrom/hecateos/install.sh"
    echo "==========================================="
fi

# Cleanup
cd ..
rm -rf "$WORK_DIR"