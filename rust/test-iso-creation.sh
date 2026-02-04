#!/bin/bash
set -e

echo "Testing HecateOS ISO Creation Process"
echo "======================================"

# Create a minimal test ISO structure
echo "1. Creating minimal test ISO content..."
mkdir -p /tmp/test-iso-content/{boot,isolinux,EFI/BOOT}
echo "Test Ubuntu ISO" > /tmp/test-iso-content/README
echo "ISOLINUX" > /tmp/test-iso-content/isolinux/isolinux.cfg
echo "EFI Boot" > /tmp/test-iso-content/EFI/BOOT/bootx64.efi

# Test native ISO creation
echo -e "\n2. Testing native ISO creation with hecate-iso..."
./target/release/hecate-iso repack /tmp/test-iso-content \
    -o /tmp/test-hecateos.iso \
    --label "TEST_HECATE"

# Check result
if [ -f /tmp/test-hecateos.iso ]; then
    echo -e "\n✅ ISO created successfully!"
    ls -lah /tmp/test-hecateos.iso
    file /tmp/test-hecateos.iso
elif [ -f /tmp/test-hecateos.tar.gz ]; then
    echo -e "\n⚠️ Fallback: tar.gz created instead of ISO"
    ls -lah /tmp/test-hecateos.tar.gz
else
    echo -e "\n❌ ISO creation failed!"
    exit 1
fi

# Test with binaries
echo -e "\n3. Testing with HecateOS binaries..."
./target/release/hecate-iso init -o /tmp/test-config.toml

# Clean up
rm -rf /tmp/test-iso-content
echo -e "\nTest complete!"