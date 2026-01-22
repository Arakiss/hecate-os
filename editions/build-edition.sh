#!/bin/bash
#
# HecateOS Multi-Edition Build System
# Generates different ISO editions based on profiles
#

set -e

# Colors
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
WHITE='\033[1;37m'
RESET='\033[0m'

# Paths
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
BASE_DIR=$(dirname "$SCRIPT_DIR")
EDITIONS_DIR="$SCRIPT_DIR"
CONFIG_DIR="$BASE_DIR/config"

# Show usage
usage() {
    echo -e "${WHITE}Usage: $0 <edition> [options]${RESET}"
    echo ""
    echo "Editions:"
    echo "  ultimate    - Full-featured for high-end AI/ML workstations"
    echo "  workstation - Professional workstation edition"
    echo "  gaming      - Optimized for gaming and streaming"
    echo "  developer   - Development-focused edition"
    echo "  lite        - Lightweight for standard hardware"
    echo "  server      - Headless server edition"
    echo "  all         - Build all editions"
    echo ""
    echo "Options:"
    echo "  --no-cache  - Don't use package cache"
    echo "  --verbose   - Verbose output"
    echo ""
    exit 1
}

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}This script must be run as root${RESET}"
        exit 1
    fi
}

# Edition: Ultimate
build_ultimate() {
    echo -e "${CYAN}Building HecateOS Ultimate Edition...${RESET}"
    
    # Create package list
    cat > "$CONFIG_DIR/package-lists/edition-ultimate.list.chroot" << 'EOF'
# Ultimate Edition Packages
# Everything for high-end workstations

# NVIDIA Full Stack
nvidia-driver-550
nvidia-cuda-toolkit
nvidia-cuda-dev
libcudnn8
libcudnn8-dev
libnccl2
libnccl-dev
tensorrt

# AI/ML Frameworks
python3-torch
python3-tensorflow-gpu
python3-jax
python3-mxnet-cuda

# Scientific Computing
python3-scipy
python3-numpy
python3-pandas
python3-matplotlib
julia
octave
r-base

# HPC Tools
openmpi-bin
libopenmpi-dev
slurm-client

# Professional Software Support
wine
wine64
libvulkan1
EOF
    
    # Custom kernel parameters
    KERNEL_PARAMS="intel_pstate=active intel_iommu=on iommu=pt nvme_core.default_ps_max_latency_us=0 pcie_aspm=off processor.max_cstate=1 intel_idle.max_cstate=0 mitigations=off transparent_hugepage=always numa_balancing=enable preempt=voluntary"
    
    # Build with ultimate profile
    build_iso "ultimate" "$KERNEL_PARAMS"
}

# Edition: Workstation
build_workstation() {
    echo -e "${CYAN}Building HecateOS Workstation Edition...${RESET}"
    
    cat > "$CONFIG_DIR/package-lists/edition-workstation.list.chroot" << 'EOF'
# Workstation Edition Packages
# Balanced for professional use

# NVIDIA Standard
nvidia-driver-550
nvidia-utils-550
nvidia-settings
nvidia-cuda-toolkit

# Development
docker.io
docker-compose
python3-pip
nodejs
npm
golang-go

# Productivity
libreoffice
thunderbird
firefox
chromium-browser
EOF
    
    KERNEL_PARAMS="quiet splash intel_pstate=active mitigations=off"
    build_iso "workstation" "$KERNEL_PARAMS"
}

# Edition: Gaming
build_gaming() {
    echo -e "${CYAN}Building HecateOS Gaming Edition...${RESET}"
    
    cat > "$CONFIG_DIR/package-lists/edition-gaming.list.chroot" << 'EOF'
# Gaming Edition Packages
# Optimized for gaming performance

# NVIDIA Gaming
nvidia-driver-550
nvidia-settings
nvidia-prime
vulkan-tools
mesa-vulkan-drivers

# Gaming Platform
steam
lutris
wine
wine64
winetricks
gamemode
mangohud

# Streaming
obs-studio
discord

# Gaming kernel tools
earlyoom
zram-config
EOF
    
    KERNEL_PARAMS="quiet splash mitigations=off preempt=full threadirqs"
    build_iso "gaming" "$KERNEL_PARAMS" "linux-lowlatency-hwe-24.04"
}

# Edition: Developer
build_developer() {
    echo -e "${CYAN}Building HecateOS Developer Edition...${RESET}"
    
    cat > "$CONFIG_DIR/package-lists/edition-developer.list.chroot" << 'EOF'
# Developer Edition Packages
# Everything for software development

# Languages
python3-full
nodejs
npm
golang-go
rustc
cargo
openjdk-17-jdk
dotnet-sdk-8.0

# Databases
postgresql
mysql-server
mongodb-org
redis

# Dev Tools
git
gh
docker.io
docker-compose
kubectl
terraform
ansible

# Editors
neovim
vim
emacs
code

# Build Tools
build-essential
cmake
ninja-build
meson
EOF
    
    KERNEL_PARAMS="quiet splash"
    build_iso "developer" "$KERNEL_PARAMS"
}

# Edition: Lite
build_lite() {
    echo -e "${CYAN}Building HecateOS Lite Edition...${RESET}"
    
    cat > "$CONFIG_DIR/package-lists/edition-lite.list.chroot" << 'EOF'
# Lite Edition Packages
# Minimal but functional

# Basic System
ubuntu-desktop-minimal
firefox
nautilus
gnome-terminal

# Essential Tools
git
curl
wget
htop
neofetch

# Basic Development
python3
python3-pip
gcc
make

# Multimedia
vlc
gimp
EOF
    
    # Remove heavy packages for lite edition
    cat > "$CONFIG_DIR/package-lists/edition-lite-remove.list.chroot" << 'EOF'
nvidia-*
cuda-*
docker*
libreoffice*
thunderbird
EOF
    
    KERNEL_PARAMS="quiet splash"
    build_iso "lite" "$KERNEL_PARAMS"
}

# Edition: Server
build_server() {
    echo -e "${CYAN}Building HecateOS Server Edition...${RESET}"
    
    cat > "$CONFIG_DIR/package-lists/edition-server.list.chroot" << 'EOF'
# Server Edition Packages
# Headless server configuration

# Server Base
ubuntu-server
openssh-server
fail2ban
ufw

# Containers
docker.io
docker-compose
containerd
podman

# Orchestration
kubectl
kubeadm
kubelet

# Monitoring
prometheus-node-exporter
grafana
netdata

# Databases
postgresql
mysql-server
redis-server

# Web Servers
nginx
apache2
certbot

# No GUI packages
!ubuntu-desktop*
!gnome-*
!x11-*
!xserver-*
!plymouth*
EOF
    
    KERNEL_PARAMS="console=tty0 console=ttyS0,115200n8"
    build_iso "server" "$KERNEL_PARAMS" "linux-server"
}

# Generic ISO build function
build_iso() {
    local EDITION=$1
    local KERNEL_PARAMS=$2
    local KERNEL_FLAVOR=${3:-"linux-generic-hwe-24.04"}
    
    echo -e "${YELLOW}Configuring build for $EDITION edition...${RESET}"
    
    cd "$BASE_DIR"
    
    # Clean previous build
    lb clean --purge 2>/dev/null || true
    
    # Configure live-build
    lb config \
        --distribution noble \
        --mode ubuntu \
        --architectures amd64 \
        --linux-flavours "$KERNEL_FLAVOR" \
        --binary-images iso-hybrid \
        --archive-areas "main restricted universe multiverse" \
        --debian-installer live \
        --debian-installer-gui false \
        --memtest none \
        --iso-application "HecateOS-$EDITION" \
        --iso-volume "HecateOS-$EDITION" \
        --iso-publisher "HecateOS Team" \
        --bootappend-live "boot=casper $KERNEL_PARAMS"
    
    # Build ISO
    echo -e "${YELLOW}Building ISO (this will take 30-60 minutes)...${RESET}"
    lb build 2>&1 | tee "build-$EDITION-$(date +%Y%m%d).log"
    
    # Move and rename ISO
    if [ -f "live-image-amd64.hybrid.iso" ]; then
        mkdir -p "$BASE_DIR/iso"
        mv "live-image-amd64.hybrid.iso" "$BASE_DIR/iso/hecate-os-$EDITION-$(date +%Y%m%d).iso"
        echo -e "${GREEN}✓ Build complete: iso/hecate-os-$EDITION-$(date +%Y%m%d).iso${RESET}"
    else
        echo -e "${RED}✗ Build failed for $EDITION edition${RESET}"
        return 1
    fi
}

# Build all editions
build_all() {
    echo -e "${CYAN}Building all HecateOS editions...${RESET}"
    
    EDITIONS="ultimate workstation gaming developer lite server"
    
    for edition in $EDITIONS; do
        echo -e "\n${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        build_$edition
    done
    
    echo -e "\n${GREEN}All editions built successfully!${RESET}"
    ls -lh "$BASE_DIR/iso/"
}

# Main execution
main() {
    check_root
    
    case "$1" in
        ultimate|workstation|gaming|developer|lite|server)
            build_$1
            ;;
        all)
            build_all
            ;;
        *)
            usage
            ;;
    esac
}

main "$@"