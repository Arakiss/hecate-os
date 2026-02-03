#!/bin/bash
#
# HecateOS VM Test Script
# Quick test of HecateOS in QEMU
#

set -e

SCRIPT_DIR="$(dirname "$0")"
ISO_PATH="$SCRIPT_DIR/output/hecateos-0.1.0-amd64.iso"

# VM Configuration
VM_MEMORY="4G"
VM_CPUS="4"
VM_DISK="hecateos-test.qcow2"
VM_DISK_SIZE="20G"

echo "==================================="
echo "  HecateOS VM Test"
echo "==================================="

# Check if ISO exists
if [ ! -f "$ISO_PATH" ]; then
    echo "Error: ISO not found at $ISO_PATH"
    echo "Please run ./build-iso.sh first"
    exit 1
fi

# Install qemu if not present
if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
    echo "Installing QEMU..."
    sudo apt-get update
    sudo apt-get install -y qemu-system-x86 qemu-utils
fi

# Create virtual disk if it doesn't exist
if [ ! -f "$VM_DISK" ]; then
    echo "Creating virtual disk..."
    qemu-img create -f qcow2 "$VM_DISK" "$VM_DISK_SIZE"
fi

echo "Starting HecateOS VM..."
echo "  Memory: $VM_MEMORY"
echo "  CPUs: $VM_CPUS"
echo "  ISO: $ISO_PATH"
echo ""
echo "VM Controls:"
echo "  - Ctrl+Alt+G: Release mouse"
echo "  - Ctrl+Alt+F: Fullscreen"
echo "  - Ctrl+Alt+2: QEMU Monitor"
echo "  - Ctrl+Alt+1: Back to VM"
echo ""

# Start VM with various options
qemu-system-x86_64 \
    -name "HecateOS Test VM" \
    -m "$VM_MEMORY" \
    -smp "$VM_CPUS" \
    -enable-kvm \
    -cpu host \
    -cdrom "$ISO_PATH" \
    -hda "$VM_DISK" \
    -boot d \
    -vga virtio \
    -display gtk \
    -netdev user,id=net0,hostfwd=tcp::9313-:9313 \
    -device virtio-net-pci,netdev=net0 \
    -usb -device usb-tablet \
    -soundhw ac97 \
    -monitor stdio

echo "VM shut down."