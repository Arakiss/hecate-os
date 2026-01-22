#!/bin/bash
#
# HecateOS Intelligent Driver Installation System
# Automatically detects and installs optimal drivers
#

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
RESET='\033[0m'

# Driver database
NVIDIA_DRIVERS=(
    "RTX 4090:550:flagship"
    "RTX 4080:550:flagship"
    "RTX 4070:550:high-end"
    "RTX 4060:550:high-end"
    "RTX 3090:535:previous-gen"
    "RTX 3080:535:previous-gen"
    "RTX 3070:535:previous-gen"
    "RTX 3060:535:previous-gen"
    "RTX 2080:525:older-gen"
    "RTX 2070:525:older-gen"
    "RTX 2060:525:older-gen"
    "GTX 1660:470:legacy"
    "GTX 1650:470:legacy"
    "GTX 1080:470:legacy"
    "GTX 1070:470:legacy"
    "GTX 1060:470:legacy"
)

AMD_DRIVERS=(
    "RX 7900:amdgpu-pro:flagship"
    "RX 7800:amdgpu-pro:high-end"
    "RX 7700:amdgpu-pro:high-end"
    "RX 7600:amdgpu:mid-range"
    "RX 6900:amdgpu-pro:previous-gen"
    "RX 6800:amdgpu-pro:previous-gen"
    "RX 6700:amdgpu:previous-gen"
    "RX 6600:amdgpu:previous-gen"
)

echo -e "${CYAN}HecateOS Intelligent Driver Installer${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Detect GPU
detect_gpu() {
    echo -e "\n${YELLOW}Detecting GPU...${RESET}"
    
    GPU_INFO=$(lspci | grep -E "VGA|3D|Display" || true)
    
    if echo "$GPU_INFO" | grep -qi nvidia; then
        GPU_VENDOR="nvidia"
        GPU_MODEL=$(echo "$GPU_INFO" | grep -i nvidia | head -1)
        
        # Try to get specific model
        if command -v nvidia-smi &> /dev/null; then
            SPECIFIC_MODEL=$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1 || true)
            if [ ! -z "$SPECIFIC_MODEL" ]; then
                GPU_MODEL="$SPECIFIC_MODEL"
            fi
        fi
        
        echo -e "${GREEN}✓${RESET} NVIDIA GPU detected: $GPU_MODEL"
        
    elif echo "$GPU_INFO" | grep -qi amd; then
        GPU_VENDOR="amd"
        GPU_MODEL=$(echo "$GPU_INFO" | grep -i amd | head -1)
        echo -e "${GREEN}✓${RESET} AMD GPU detected: $GPU_MODEL"
        
    elif echo "$GPU_INFO" | grep -qi intel; then
        GPU_VENDOR="intel"
        GPU_MODEL=$(echo "$GPU_INFO" | grep -i intel | head -1)
        echo -e "${GREEN}✓${RESET} Intel GPU detected: $GPU_MODEL"
        
    else
        GPU_VENDOR="unknown"
        echo -e "${YELLOW}⚠${RESET} No dedicated GPU detected"
        return 1
    fi
    
    return 0
}

# Select NVIDIA driver
select_nvidia_driver() {
    local model="$1"
    local driver_version="550"  # Default to latest
    local tier="unknown"
    
    # Check against known models
    for entry in "${NVIDIA_DRIVERS[@]}"; do
        IFS=':' read -r known_model version gpu_tier <<< "$entry"
        if echo "$model" | grep -qi "$known_model"; then
            driver_version="$version"
            tier="$gpu_tier"
            echo -e "${GREEN}✓${RESET} Matched: $known_model (Driver $version, Tier: $tier)"
            break
        fi
    done
    
    # If no exact match, try to determine by series
    if [ "$tier" == "unknown" ]; then
        if echo "$model" | grep -qi "RTX 40"; then
            driver_version="550"
            tier="current-gen"
        elif echo "$model" | grep -qi "RTX 30"; then
            driver_version="535"
            tier="previous-gen"
        elif echo "$model" | grep -qi "RTX 20"; then
            driver_version="525"
            tier="older-gen"
        elif echo "$model" | grep -qi "GTX"; then
            driver_version="470"
            tier="legacy"
        fi
    fi
    
    echo "$driver_version:$tier"
}

# Install NVIDIA driver
install_nvidia_driver() {
    local driver_version="$1"
    local tier="$2"
    
    echo -e "\n${CYAN}Installing NVIDIA Driver $driver_version...${RESET}"
    
    # Add NVIDIA PPA if needed
    if ! grep -q "graphics-drivers" /etc/apt/sources.list.d/*.list 2>/dev/null; then
        echo "Adding NVIDIA PPA..."
        add-apt-repository -y ppa:graphics-drivers/ppa
        apt update
    fi
    
    # Determine packages to install based on tier
    case "$tier" in
        flagship|high-end)
            PACKAGES="nvidia-driver-$driver_version nvidia-dkms-$driver_version nvidia-utils-$driver_version nvidia-settings nvidia-cuda-toolkit"
            ;;
        previous-gen|older-gen)
            PACKAGES="nvidia-driver-$driver_version nvidia-dkms-$driver_version nvidia-utils-$driver_version nvidia-settings"
            ;;
        legacy)
            PACKAGES="nvidia-driver-$driver_version nvidia-settings"
            ;;
        *)
            PACKAGES="nvidia-driver-$driver_version"
            ;;
    esac
    
    # Remove old drivers if present
    echo "Removing old NVIDIA drivers if present..."
    apt purge -y nvidia-* 2>/dev/null || true
    
    # Install new driver
    echo "Installing NVIDIA driver packages..."
    apt install -y $PACKAGES
    
    # Configure driver
    echo "Configuring NVIDIA driver..."
    
    # Blacklist nouveau
    cat > /etc/modprobe.d/blacklist-nouveau.conf << EOF
blacklist nouveau
options nouveau modeset=0
EOF
    
    # Update initramfs
    update-initramfs -u
    
    # Enable persistence daemon
    systemctl enable nvidia-persistenced || true
    
    echo -e "${GREEN}✓${RESET} NVIDIA Driver $driver_version installed successfully"
    
    # Apply optimizations based on tier
    if [ "$tier" == "flagship" ] || [ "$tier" == "high-end" ]; then
        echo -e "\n${CYAN}Applying high-performance optimizations...${RESET}"
        nvidia-smi -pm 1 2>/dev/null || true
        nvidia-smi -pl 450 2>/dev/null || true
        echo -e "${GREEN}✓${RESET} Performance optimizations applied"
    fi
}

# Install AMD driver
install_amd_driver() {
    local model="$1"
    local driver_type="amdgpu"  # Default open-source driver
    
    echo -e "\n${CYAN}Installing AMD Driver...${RESET}"
    
    # Check for specific models that benefit from PRO driver
    for entry in "${AMD_DRIVERS[@]}"; do
        IFS=':' read -r known_model driver tier <<< "$entry"
        if echo "$model" | grep -qi "$known_model"; then
            driver_type="$driver"
            echo -e "${GREEN}✓${RESET} Matched: $known_model (Driver: $driver)"
            break
        fi
    done
    
    if [ "$driver_type" == "amdgpu-pro" ]; then
        echo "Installing AMD GPU PRO driver..."
        
        # Download and install AMDGPU-PRO
        AMDGPU_URL="https://drivers.amd.com/drivers/linux/amdgpu-pro-22.40-1538782-ubuntu-22.04.tar.xz"
        wget -q --show-progress "$AMDGPU_URL" -O /tmp/amdgpu-pro.tar.xz
        tar -xf /tmp/amdgpu-pro.tar.xz -C /tmp/
        /tmp/amdgpu-pro-*/amdgpu-pro-install -y --opencl=rocr,legacy
        
        echo -e "${GREEN}✓${RESET} AMD GPU PRO driver installed"
    else
        echo "Using open-source AMDGPU driver..."
        
        # Install firmware and mesa packages
        apt install -y firmware-amd-graphics mesa-vulkan-drivers mesa-vdpau-drivers
        
        # Enable AMDGPU driver
        echo "options amdgpu si_support=1" > /etc/modprobe.d/amdgpu.conf
        echo "options amdgpu cik_support=1" >> /etc/modprobe.d/amdgpu.conf
        
        update-initramfs -u
        
        echo -e "${GREEN}✓${RESET} AMDGPU driver configured"
    fi
}

# Install Intel driver
install_intel_driver() {
    echo -e "\n${CYAN}Installing Intel Graphics Driver...${RESET}"
    
    # Install Intel graphics packages
    apt install -y intel-media-va-driver i965-va-driver vainfo mesa-vulkan-drivers
    
    # Enable GuC/HuC firmware for better performance
    echo "options i915 enable_guc=3" > /etc/modprobe.d/i915.conf
    
    update-initramfs -u
    
    echo -e "${GREEN}✓${RESET} Intel graphics driver installed"
}

# Install additional tools
install_gpu_tools() {
    echo -e "\n${CYAN}Installing GPU management tools...${RESET}"
    
    TOOLS="mesa-utils vulkan-tools vainfo vdpauinfo clinfo"
    
    if [ "$GPU_VENDOR" == "nvidia" ]; then
        TOOLS="$TOOLS nvtop gpustat"
    elif [ "$GPU_VENDOR" == "amd" ]; then
        TOOLS="$TOOLS radeontop"
    fi
    
    apt install -y $TOOLS 2>/dev/null || true
    
    echo -e "${GREEN}✓${RESET} GPU tools installed"
}

# Verify installation
verify_installation() {
    echo -e "\n${CYAN}Verifying driver installation...${RESET}"
    
    case "$GPU_VENDOR" in
        nvidia)
            if nvidia-smi &> /dev/null; then
                echo -e "${GREEN}✓${RESET} NVIDIA driver is working"
                nvidia-smi
            else
                echo -e "${RED}✗${RESET} NVIDIA driver verification failed"
                echo -e "${YELLOW}A reboot may be required${RESET}"
                return 1
            fi
            ;;
        
        amd)
            if lsmod | grep -q amdgpu; then
                echo -e "${GREEN}✓${RESET} AMD driver is loaded"
                glxinfo | grep "OpenGL renderer" || true
            else
                echo -e "${YELLOW}⚠${RESET} AMD driver not loaded yet"
                echo -e "${YELLOW}A reboot is required${RESET}"
            fi
            ;;
        
        intel)
            if glxinfo | grep -qi intel; then
                echo -e "${GREEN}✓${RESET} Intel driver is working"
                glxinfo | grep "OpenGL renderer" || true
            else
                echo -e "${YELLOW}⚠${RESET} Intel driver verification pending"
            fi
            ;;
    esac
    
    # Test Vulkan
    if command -v vulkaninfo &> /dev/null; then
        echo -e "\n${CYAN}Vulkan support:${RESET}"
        vulkaninfo --summary 2>/dev/null | head -10 || echo "  Not available"
    fi
}

# Main installation flow
main() {
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}This script must be run as root${RESET}"
        exit 1
    fi
    
    # Detect GPU
    if ! detect_gpu; then
        echo -e "${YELLOW}No GPU driver installation needed${RESET}"
        exit 0
    fi
    
    # Update package lists
    echo -e "\n${CYAN}Updating package lists...${RESET}"
    apt update
    
    # Install driver based on vendor
    case "$GPU_VENDOR" in
        nvidia)
            # Select optimal driver
            driver_info=$(select_nvidia_driver "$GPU_MODEL")
            driver_version=$(echo "$driver_info" | cut -d: -f1)
            tier=$(echo "$driver_info" | cut -d: -f2)
            
            echo -e "\n${WHITE}Selected Driver:${RESET} NVIDIA $driver_version"
            echo -e "${WHITE}GPU Tier:${RESET} $tier"
            
            # Confirm installation
            read -p "Proceed with installation? (Y/n): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Nn]$ ]]; then
                install_nvidia_driver "$driver_version" "$tier"
            fi
            ;;
        
        amd)
            install_amd_driver "$GPU_MODEL"
            ;;
        
        intel)
            install_intel_driver
            ;;
        
        *)
            echo -e "${RED}Unsupported GPU vendor${RESET}"
            exit 1
            ;;
    esac
    
    # Install additional tools
    install_gpu_tools
    
    # Verify installation
    verify_installation
    
    # Save driver info
    mkdir -p /etc/hecate
    cat > /etc/hecate/gpu-driver.json << EOF
{
    "vendor": "$GPU_VENDOR",
    "model": "$GPU_MODEL",
    "driver": "${driver_version:-default}",
    "tier": "${tier:-unknown}",
    "installed": "$(date -Iseconds)"
}
EOF
    
    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${GREEN}Driver installation complete!${RESET}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    
    if [ "$GPU_VENDOR" == "nvidia" ] || [ "$GPU_VENDOR" == "amd" ]; then
        echo -e "\n${YELLOW}A reboot is recommended to ensure drivers are fully loaded${RESET}"
        echo -e "Run '${CYAN}sudo reboot${RESET}' when ready"
    fi
}

# Handle arguments
case "$1" in
    --detect-only)
        detect_gpu
        ;;
    --verify)
        GPU_VENDOR=$(jq -r '.vendor' /etc/hecate/gpu-driver.json 2>/dev/null || echo "unknown")
        verify_installation
        ;;
    --help)
        echo "Usage: $0 [options]"
        echo "Options:"
        echo "  --detect-only  Only detect GPU, don't install"
        echo "  --verify       Verify existing driver installation"
        echo "  --help         Show this help"
        ;;
    *)
        main
        ;;
esac