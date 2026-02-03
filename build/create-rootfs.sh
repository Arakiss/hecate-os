#!/bin/bash
#
# HecateOS Root Filesystem Creator
# Creates minimal rootfs with HecateOS components
#

set -e

SCRIPT_DIR="$(dirname "$0")"
ROOTFS_DIR="$SCRIPT_DIR/rootfs"
RUST_DIR="$(dirname "$SCRIPT_DIR")/rust"
OUTPUT_DIR="$SCRIPT_DIR/output"

echo "==================================="
echo "  HecateOS RootFS Creation"
echo "==================================="

# Cleanup and create directories
rm -rf "$ROOTFS_DIR"
mkdir -p "$ROOTFS_DIR"
cd "$ROOTFS_DIR"

# Create base directory structure
echo "Creating base directories..."
mkdir -p {bin,sbin,etc,dev,lib,lib64,mnt,opt,proc,root,run,srv,sys,tmp,usr,var}
mkdir -p usr/{bin,sbin,lib,share}
mkdir -p var/{log,cache,lib,run}
mkdir -p etc/{hecate,systemd/system,skel}

# Create essential device nodes (will be properly created by devtmpfs)
echo "Creating device nodes..."
mkdir -p dev
mknod -m 600 dev/console c 5 1 2>/dev/null || true
mknod -m 666 dev/null c 1 3 2>/dev/null || true

# Install busybox for basic utilities
echo "Installing busybox..."
if command -v busybox >/dev/null 2>&1; then
    cp $(which busybox) bin/
    # Create symlinks for busybox applets
    for app in sh ash bash cat ls cp mv rm mkdir rmdir echo pwd \
                grep sed awk cut tr sort uniq head tail less more \
                mount umount df du ps top kill killall ping wget \
                tar gzip gunzip xz unxz; do
        ln -sf busybox bin/$app 2>/dev/null || true
    done
else
    echo "Warning: busybox not found, downloading static binary..."
    wget -O bin/busybox https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox
    chmod +x bin/busybox
    bin/busybox --install -s bin/
fi

# Create init script
echo "Creating init system..."
cat > init << 'EOF'
#!/bin/sh
#
# HecateOS Init Script
#

echo "Starting HecateOS..."

# Mount essential filesystems
/bin/mount -t proc none /proc
/bin/mount -t sysfs none /sys
/bin/mount -t devtmpfs none /dev
/bin/mount -t tmpfs none /tmp
/bin/mount -t tmpfs none /run

# Setup networking
/sbin/ip link set lo up

# Load kernel modules
if [ -d /lib/modules ]; then
    echo "Loading kernel modules..."
    find /lib/modules -name "*.ko" -exec insmod {} \; 2>/dev/null
fi

# Start HecateOS services
echo "Starting HecateOS services..."

# Start system daemon
if [ -x /usr/bin/hecated ]; then
    echo "Starting HecateOS daemon..."
    /usr/bin/hecated &
fi

# Start monitoring server
if [ -x /usr/bin/hecate-monitor ]; then
    echo "Starting monitoring server..."
    /usr/bin/hecate-monitor &
fi

# Print system information
echo ""
echo "==================================="
echo "   Welcome to HecateOS v0.1.0"
echo "==================================="
echo ""
if [ -x /usr/bin/hecate-bench ]; then
    /usr/bin/hecate-bench sysinfo
fi

# Start shell
echo ""
echo "Starting shell..."
exec /bin/sh
EOF
chmod +x init

# Install HecateOS components
echo "Installing HecateOS components..."
if [ -d "$RUST_DIR/target/release" ]; then
    # Core binaries
    for binary in hecated hecate-monitor hecate-bench hecate-pkg \
                  hecate-dev hecate-lint hecate-sign; do
        if [ -f "$RUST_DIR/target/release/$binary" ]; then
            echo "  Installing $binary..."
            cp "$RUST_DIR/target/release/$binary" usr/bin/
            # Strip debug symbols for size
            strip usr/bin/$binary 2>/dev/null || true
        fi
    done
else
    echo "Warning: Rust binaries not found. Building them first..."
    (cd "$RUST_DIR" && cargo build --release)
    # Retry installation
    for binary in hecated hecate-monitor hecate-bench hecate-pkg; do
        if [ -f "$RUST_DIR/target/release/$binary" ]; then
            cp "$RUST_DIR/target/release/$binary" usr/bin/
            strip usr/bin/$binary 2>/dev/null || true
        fi
    done
fi

# Create basic configuration files
echo "Creating configuration..."

# /etc/passwd
cat > etc/passwd << 'EOF'
root:x:0:0:root:/root:/bin/sh
daemon:x:1:1:daemon:/usr/sbin:/bin/false
nobody:x:65534:65534:nobody:/nonexistent:/bin/false
EOF

# /etc/group
cat > etc/group << 'EOF'
root:x:0:
daemon:x:1:
nobody:x:65534:
EOF

# /etc/hostname
echo "hecate" > etc/hostname

# /etc/hosts
cat > etc/hosts << 'EOF'
127.0.0.1   localhost
127.0.1.1   hecate
EOF

# /etc/fstab
cat > etc/fstab << 'EOF'
# <file system> <mount point>   <type>  <options>       <dump>  <pass>
proc            /proc           proc    defaults        0       0
sysfs           /sys            sysfs   defaults        0       0
devtmpfs        /dev            devtmpfs defaults       0       0
tmpfs           /tmp            tmpfs   defaults        0       0
tmpfs           /run            tmpfs   defaults        0       0
EOF

# HecateOS configuration
cat > etc/hecate/system.conf << 'EOF'
# HecateOS System Configuration
[system]
version = "0.1.0"
hostname = "hecate"
optimization_level = "aggressive"

[performance]
cpu_governor = "performance"
io_scheduler = "bfq"
transparent_hugepages = "always"

[monitoring]
enabled = true
port = 9313
interval = 1
EOF

# Create libraries manifest
echo "Collecting required libraries..."
mkdir -p lib/x86_64-linux-gnu lib64

# Function to copy library and its dependencies
copy_libs() {
    local binary=$1
    if [ -f "$binary" ]; then
        for lib in $(ldd "$binary" 2>/dev/null | grep -E '(/lib|/usr/lib)' | awk '{print $3}'); do
            if [ -f "$lib" ] && [ ! -f ".$lib" ]; then
                echo "  Copying $lib..."
                cp -L "$lib" ".$lib"
            fi
        done
    fi
}

# Copy libraries for our binaries
for binary in usr/bin/hecate*; do
    copy_libs "$binary"
done

# Copy basic system libraries
for lib in /lib/x86_64-linux-gnu/lib{c,m,pthread,dl,rt,util}.so* \
           /lib64/ld-linux-x86-64.so*; do
    if [ -f "$lib" ]; then
        cp -L "$lib" ".$(dirname $lib)/" 2>/dev/null || true
    fi
done

# Create rootfs archive
echo "Creating rootfs archive..."
find . | cpio -o -H newc | xz -9 > "$OUTPUT_DIR/rootfs.cpio.xz"

echo "RootFS created: $OUTPUT_DIR/rootfs.cpio.xz"
echo "Size: $(du -h "$OUTPUT_DIR/rootfs.cpio.xz" | cut -f1)"