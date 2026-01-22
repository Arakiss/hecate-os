#!/usr/bin/env bash
#
# HecateOS Web Installer
# curl -fsSL hecate.sh | bash
# or
# wget -qO- hecate.sh | bash
#

set -e

# Colors
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
RESET='\033[0m'

# ASCII Logo
show_logo() {
    echo -e "${PURPLE}"
    cat << 'EOF'
    ╦ ╦┌─┐┌─┐┌─┐┌┬┐┌─┐╔═╗╔═╗
    ╠═╣├┤ │  ├─┤ │ ├┤ ║ ║╚═╗
    ╩ ╩└─┘└─┘┴ ┴ ┴ └─┘╚═╝╚═╝
EOF
    echo -e "${RESET}"
    echo -e "${CYAN}Adaptive Linux for Any Hardware${RESET}"
    echo ""
}

# Detect system
detect_system() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        VER=$VERSION_ID
    fi
    
    if [[ "$OS" != "ubuntu" ]]; then
        echo -e "${YELLOW}⚠ Warning: HecateOS is designed for Ubuntu 24.04${RESET}"
        echo -e "${YELLOW}  Current OS: $OS $VER${RESET}"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
    fi
}

# Quick hardware check
quick_check() {
    echo -e "${CYAN}Quick Hardware Check:${RESET}"
    
    # CPU
    CPU=$(lscpu | grep "Model name" | cut -d: -f2 | xargs)
    echo -e "  CPU: ${GREEN}$CPU${RESET}"
    
    # RAM
    RAM=$(free -h | grep Mem | awk '{print $2}')
    echo -e "  RAM: ${GREEN}$RAM${RESET}"
    
    # GPU
    if lspci | grep -q -i nvidia; then
        GPU="NVIDIA detected"
    elif lspci | grep -q -i amd; then
        GPU="AMD detected"
    else
        GPU="Integrated graphics"
    fi
    echo -e "  GPU: ${GREEN}$GPU${RESET}"
    
    echo ""
}

# Installation menu
show_menu() {
    echo -e "${CYAN}What would you like to do?${RESET}"
    echo ""
    echo "  1) Download HecateOS ISO (choose edition)"
    echo "  2) Optimize current Ubuntu installation"
    echo "  3) Install HecateOS tools only"
    echo "  4) Hardware detection test"
    echo "  5) Exit"
    echo ""
    read -p "Select option [1-5]: " -n 1 -r choice
    echo ""
}

# Download ISO
download_iso() {
    echo -e "\n${CYAN}Select HecateOS Edition:${RESET}"
    echo "  1) Ultimate (AI/ML, 64GB+ RAM, RTX 4080+)"
    echo "  2) Workstation (Professional, 32GB+ RAM)"
    echo "  3) Gaming (Gaming optimized, 16GB+ RAM)"
    echo "  4) Developer (Coding focused, 16GB+ RAM)"
    echo "  5) Lite (Standard hardware, 8GB+ RAM)"
    echo "  6) Server (Headless, no GUI)"
    echo ""
    read -p "Select edition [1-6]: " -n 1 -r edition
    echo ""
    
    case $edition in
        1) EDITION="ultimate" ;;
        2) EDITION="workstation" ;;
        3) EDITION="gaming" ;;
        4) EDITION="developer" ;;
        5) EDITION="lite" ;;
        6) EDITION="server" ;;
        *) echo "Invalid selection"; exit 1 ;;
    esac
    
    ISO_URL="https://github.com/Arakiss/hecate-os/releases/latest/download/hecate-os-$EDITION-latest.iso"
    
    echo -e "${CYAN}Downloading HecateOS $EDITION edition...${RESET}"
    echo -e "${YELLOW}URL: $ISO_URL${RESET}"
    
    if command -v wget > /dev/null; then
        wget --show-progress -O "hecate-os-$EDITION.iso" "$ISO_URL"
    elif command -v curl > /dev/null; then
        curl -L --progress-bar -o "hecate-os-$EDITION.iso" "$ISO_URL"
    else
        echo -e "${RED}Neither wget nor curl found!${RESET}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Downloaded: hecate-os-$EDITION.iso${RESET}"
    echo -e "\n${CYAN}Create bootable USB:${RESET}"
    echo -e "  sudo dd if=hecate-os-$EDITION.iso of=/dev/sdX bs=4M status=progress"
}

# Optimize current system
optimize_system() {
    echo -e "\n${CYAN}Downloading HecateOS optimization scripts...${RESET}"
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Download scripts
    SCRIPTS=(
        "hardware-detector.sh"
        "apply-optimizations.sh"
        "hecate-driver-installer.sh"
    )
    
    for script in "${SCRIPTS[@]}"; do
        echo -e "Downloading $script..."
        curl -fsSL "https://raw.githubusercontent.com/Arakiss/hecate-os/main/scripts/$script" -o "$script"
        chmod +x "$script"
    done
    
    echo -e "\n${YELLOW}This will optimize your current Ubuntu installation.${RESET}"
    echo -e "${YELLOW}Changes can be reverted but backup important data first!${RESET}"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo ./hardware-detector.sh
        sudo ./apply-optimizations.sh
        
        echo -e "\n${GREEN}✓ System optimized!${RESET}"
        echo -e "${CYAN}Reboot recommended for full effect.${RESET}"
    fi
    
    # Cleanup
    cd ~
    rm -rf "$TEMP_DIR"
}

# Install tools only
install_tools() {
    echo -e "\n${CYAN}Installing HecateOS tools...${RESET}"
    
    # Download and install tools
    sudo mkdir -p /usr/local/bin
    
    TOOLS=(
        "hecate-info"
        "hecate-benchmark.sh"
        "hardware-detector.sh"
    )
    
    for tool in "${TOOLS[@]}"; do
        echo "Installing $tool..."
        sudo curl -fsSL "https://raw.githubusercontent.com/Arakiss/hecate-os/main/scripts/$tool" \
             -o "/usr/local/bin/${tool%.sh}"
        sudo chmod +x "/usr/local/bin/${tool%.sh}"
    done
    
    echo -e "${GREEN}✓ HecateOS tools installed!${RESET}"
    echo -e "Run '${CYAN}hecate-info${RESET}' to test"
}

# Hardware test
hardware_test() {
    echo -e "\n${CYAN}Running hardware detection test...${RESET}"
    
    # Download and run hardware detector
    curl -fsSL "https://raw.githubusercontent.com/Arakiss/hecate-os/main/scripts/hardware-detector.sh" | sudo bash
}

# Main
main() {
    clear
    show_logo
    detect_system
    quick_check
    show_menu
    
    case $choice in
        1) download_iso ;;
        2) optimize_system ;;
        3) install_tools ;;
        4) hardware_test ;;
        5) echo "Goodbye!" ; exit 0 ;;
        *) echo "Invalid option" ; exit 1 ;;
    esac
    
    echo -e "\n${GREEN}Done!${RESET}"
    echo -e "${CYAN}Visit https://hecateos.dev for more information${RESET}"
}

# Run
main "$@"