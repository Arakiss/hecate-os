#!/bin/bash
# Create a minimal test ISO to demonstrate the tool

set -e

echo "Creating minimal test ISO..."

# Create directory structure
mkdir -p test-iso/{isolinux,boot/grub,.disk}

# Create a minimal isolinux config
cat > test-iso/isolinux/isolinux.cfg << 'EOF'
DEFAULT test
LABEL test
  menu label Test Entry
  kernel /casper/vmlinuz
  append boot=test
EOF

# Create disk info
echo "Test ISO for HecateOS Builder" > test-iso/.disk/info

# Create a README
cat > test-iso/README << 'EOF'
This is a minimal test ISO for demonstrating the HecateOS ISO Builder.
It's not bootable but serves to test the extraction and modification process.
EOF

# Create the ISO using genisoimage or mkisofs
if command -v genisoimage >/dev/null 2>&1; then
    TOOL="genisoimage"
elif command -v mkisofs >/dev/null 2>&1; then
    TOOL="mkisofs"
else
    echo "No ISO creation tool found. Using tar archive instead..."
    tar czf test-ubuntu.iso test-iso/
    echo "Created test-ubuntu.iso (tar archive)"
    rm -rf test-iso
    exit 0
fi

$TOOL -r -J -o test-ubuntu.iso -V "TEST_ISO" test-iso/ 2>/dev/null || {
    echo "ISO creation failed, creating tar archive instead..."
    tar czf test-ubuntu.iso test-iso/
}

rm -rf test-iso
echo "Created test-ubuntu.iso"