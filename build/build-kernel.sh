#!/bin/bash
#
# HecateOS Kernel Build Script
# Builds optimized Linux kernel for HecateOS
#

set -e

KERNEL_VERSION="6.7.2"
KERNEL_URL="https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-${KERNEL_VERSION}.tar.xz"
BUILD_DIR="$(dirname "$0")/kernel-build"
OUTPUT_DIR="$(dirname "$0")/output"

echo "==================================="
echo "  HecateOS Kernel Build"
echo "==================================="

# Create directories
mkdir -p "$BUILD_DIR" "$OUTPUT_DIR"
cd "$BUILD_DIR"

# Download kernel if needed
if [ ! -f "linux-${KERNEL_VERSION}.tar.xz" ]; then
    echo "Downloading Linux kernel ${KERNEL_VERSION}..."
    wget "$KERNEL_URL"
fi

# Extract kernel
if [ ! -d "linux-${KERNEL_VERSION}" ]; then
    echo "Extracting kernel..."
    tar -xf "linux-${KERNEL_VERSION}.tar.xz"
fi

cd "linux-${KERNEL_VERSION}"

# Create optimized config
echo "Configuring kernel..."
cat > .config << 'EOF'
# HecateOS Optimized Kernel Config
CONFIG_LOCALVERSION="-hecate"
CONFIG_DEFAULT_HOSTNAME="hecate"

# Core options
CONFIG_64BIT=y
CONFIG_SMP=y
CONFIG_PREEMPT_VOLUNTARY=y
CONFIG_NO_HZ_FULL=y
CONFIG_HIGH_RES_TIMERS=y

# CPU optimizations
CONFIG_PROCESSOR_SELECT=y
CONFIG_CPU_SUP_INTEL=y
CONFIG_CPU_SUP_AMD=y
CONFIG_MCORE2=y
CONFIG_X86_MSR=y
CONFIG_X86_CPUID=y
CONFIG_CPU_FREQ=y
CONFIG_CPU_FREQ_GOV_PERFORMANCE=y
CONFIG_CPU_FREQ_GOV_ONDEMAND=y

# Memory optimizations
CONFIG_TRANSPARENT_HUGEPAGE=y
CONFIG_TRANSPARENT_HUGEPAGE_ALWAYS=y
CONFIG_ZSWAP=y
CONFIG_Z3FOLD=y
CONFIG_ZSMALLOC=y

# I/O optimizations
CONFIG_IOSCHED_BFQ=y
CONFIG_BFQ_GROUP_IOSCHED=y
CONFIG_SCSI_MQ_DEFAULT=y

# Filesystem support
CONFIG_EXT4_FS=y
CONFIG_BTRFS_FS=y
CONFIG_XFS_FS=y
CONFIG_F2FS_FS=y
CONFIG_OVERLAY_FS=y

# Networking
CONFIG_NET=y
CONFIG_INET=y
CONFIG_IPV6=y
CONFIG_NETFILTER=y
CONFIG_TCP_CONG_BBR=y
CONFIG_DEFAULT_BBR=y

# Virtualization detection
CONFIG_HYPERVISOR_GUEST=y
CONFIG_KVM_GUEST=y
CONFIG_VIRTIO=y
CONFIG_VIRTIO_PCI=y
CONFIG_VIRTIO_BLK=y
CONFIG_VIRTIO_NET=y

# Graphics
CONFIG_DRM=y
CONFIG_DRM_I915=y
CONFIG_DRM_AMDGPU=y
CONFIG_DRM_NOUVEAU=y
CONFIG_FB=y
CONFIG_FRAMEBUFFER_CONSOLE=y

# USB support
CONFIG_USB=y
CONFIG_USB_XHCI_HCD=y
CONFIG_USB_EHCI_HCD=y
CONFIG_USB_STORAGE=y

# Security
CONFIG_SECURITY=y
CONFIG_SECURITY_SELINUX=y
CONFIG_SECURITY_APPARMOR=y
CONFIG_SECURITY_LOADPIN=y
CONFIG_HARDENED_USERCOPY=y

# Compression
CONFIG_KERNEL_XZ=y
CONFIG_RD_XZ=y
CONFIG_INITRAMFS_COMPRESSION_XZ=y

# Modules
CONFIG_MODULES=y
CONFIG_MODULE_COMPRESS=y
CONFIG_MODULE_COMPRESS_XZ=y

# Debug options (disabled for performance)
# CONFIG_DEBUG_KERNEL is not set
# CONFIG_FTRACE is not set
# CONFIG_KPROBES is not set
EOF

# Expand config
make olddefconfig

# Build kernel
echo "Building kernel (this will take a while)..."
make -j$(nproc) bzImage modules

# Install to output
echo "Installing kernel..."
cp arch/x86/boot/bzImage "$OUTPUT_DIR/vmlinuz-hecate"
cp System.map "$OUTPUT_DIR/System.map-hecate"
cp .config "$OUTPUT_DIR/kernel-config"

# Build modules
echo "Installing modules..."
INSTALL_MOD_PATH="$OUTPUT_DIR/modules" make modules_install

echo "Kernel build complete!"
echo "Output: $OUTPUT_DIR/vmlinuz-hecate"