#!/bin/bash
#
# HecateOS Installer Script
# This script installs HecateOS components on an Ubuntu system
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Installation directories
INSTALL_PREFIX="/usr/local"
BIN_DIR="${INSTALL_PREFIX}/bin"
LIB_DIR="${INSTALL_PREFIX}/lib/hecate"
CONFIG_DIR="/etc/hecate"
SYSTEMD_DIR="/etc/systemd/system"

# Component list
COMPONENTS=(
    "hecate:Main CLI interface"
    "hecated:System optimization daemon"
    "hecate-monitor:Performance monitoring service"
    "hecate-bench:Benchmarking tool"
    "hecate-pkg:Package manager"
    "hecate-dev:Development tools"
    "hecate-iso:ISO builder"
)

# Print header
print_header() {
    echo -e "${CYAN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║       HecateOS Installation Script         ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════╝${NC}"
    echo
}

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}Error: This script must be run as root${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

# Check Ubuntu version
check_ubuntu() {
    if [ ! -f /etc/os-release ]; then
        echo -e "${YELLOW}Warning: Cannot detect OS version${NC}"
        return
    fi
    
    . /etc/os-release
    if [[ ! "$ID" == "ubuntu" ]]; then
        echo -e "${YELLOW}Warning: This is not Ubuntu. HecateOS is designed for Ubuntu.${NC}"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    echo -e "${GREEN}✓${NC} Detected: $PRETTY_NAME"
}

# Detect hardware
detect_hardware() {
    echo -e "\n${BLUE}Hardware Detection:${NC}"
    echo "══════════════════════════════════════════════"
    
    # CPU
    CPU_MODEL=$(lscpu | grep "Model name" | cut -d: -f2 | xargs)
    CPU_CORES=$(nproc)
    echo -e "  CPU: ${CPU_MODEL} (${CPU_CORES} cores)"
    
    # Memory
    MEM_TOTAL=$(free -h | grep Mem | awk '{print $2}')
    echo -e "  RAM: ${MEM_TOTAL}"
    
    # GPU
    if command -v lspci &> /dev/null; then
        GPU=$(lspci | grep -E "VGA|3D" | cut -d: -f3 | head -1 | xargs)
        if [ ! -z "$GPU" ]; then
            echo -e "  GPU: ${GPU}"
        fi
    fi
    
    # Disk
    DISK_TOTAL=$(df -h / | tail -1 | awk '{print $2}')
    DISK_AVAIL=$(df -h / | tail -1 | awk '{print $4}')
    echo -e "  Disk: ${DISK_AVAIL} available of ${DISK_TOTAL}"
}

# Install binaries
install_binaries() {
    echo -e "\n${BLUE}Installing HecateOS Components:${NC}"
    echo "══════════════════════════════════════════════"
    
    # Create directories
    mkdir -p "$BIN_DIR" "$LIB_DIR" "$CONFIG_DIR"
    
    # Find the binaries directory
    if [ -d "/hecateos/bin" ]; then
        BIN_SOURCE="/hecateos/bin"
    elif [ -d "./bin" ]; then
        BIN_SOURCE="./bin"
    elif [ -d "../target/release" ]; then
        BIN_SOURCE="../target/release"
    else
        echo -e "${RED}Error: Cannot find HecateOS binaries${NC}"
        echo "Expected location: /hecateos/bin or ./bin"
        exit 1
    fi
    
    # Install each component
    for comp_desc in "${COMPONENTS[@]}"; do
        IFS=':' read -r comp desc <<< "$comp_desc"
        
        if [ -f "$BIN_SOURCE/$comp" ]; then
            echo -ne "  Installing ${comp}..."
            cp "$BIN_SOURCE/$comp" "$BIN_DIR/"
            chmod 755 "$BIN_DIR/$comp"
            echo -e " ${GREEN}✓${NC}"
        else
            echo -e "  ${YELLOW}⚠${NC} ${comp} not found (optional)"
        fi
    done
}

# Install systemd services
install_services() {
    echo -e "\n${BLUE}Installing System Services:${NC}"
    echo "══════════════════════════════════════════════"
    
    # hecated service
    cat > "$SYSTEMD_DIR/hecated.service" << EOF
[Unit]
Description=HecateOS System Optimization Daemon
After=network.target

[Service]
Type=simple
ExecStart=$BIN_DIR/hecated
Restart=on-failure
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
EOF
    echo -e "  Created hecated.service ${GREEN}✓${NC}"
    
    # hecate-monitor service
    cat > "$SYSTEMD_DIR/hecate-monitor.service" << EOF
[Unit]
Description=HecateOS Performance Monitor
After=network.target

[Service]
Type=simple
ExecStart=$BIN_DIR/hecate-monitor
Restart=on-failure
RestartSec=5
User=root
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
EOF
    echo -e "  Created hecate-monitor.service ${GREEN}✓${NC}"
    
    # Reload systemd
    systemctl daemon-reload
}

# Configure system
configure_system() {
    echo -e "\n${BLUE}Configuring System:${NC}"
    echo "══════════════════════════════════════════════"
    
    # Create default config
    cat > "$CONFIG_DIR/hecate.toml" << EOF
# HecateOS Configuration
[system]
auto_optimize = true
monitoring_enabled = true

[performance]
profile = "balanced"  # balanced, performance, powersave

[gpu]
auto_configure = true
multi_gpu_support = true

[ml]
framework_optimization = true
auto_batch_sizing = true
EOF
    echo -e "  Created configuration ${GREEN}✓${NC}"
    
    # Add to PATH if not already there
    if ! grep -q "$BIN_DIR" /etc/environment; then
        sed -i "s|PATH=\"|PATH=\"$BIN_DIR:|" /etc/environment
        echo -e "  Updated PATH ${GREEN}✓${NC}"
    fi
}

# Enable services
enable_services() {
    echo -e "\n${BLUE}Enabling Services:${NC}"
    echo "══════════════════════════════════════════════"
    
    read -p "Enable hecated service? (Y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        systemctl enable hecated.service
        systemctl start hecated.service
        echo -e "  hecated service ${GREEN}enabled and started${NC}"
    fi
    
    read -p "Enable hecate-monitor service? (Y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        systemctl enable hecate-monitor.service
        systemctl start hecate-monitor.service
        echo -e "  hecate-monitor service ${GREEN}enabled and started${NC}"
        echo -e "  ${CYAN}Dashboard available at: http://localhost:9313${NC}"
    fi
}

# Print summary
print_summary() {
    echo -e "\n${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     HecateOS Installation Complete!        ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    echo
    echo "Installed components:"
    for comp_desc in "${COMPONENTS[@]}"; do
        IFS=':' read -r comp desc <<< "$comp_desc"
        if [ -f "$BIN_DIR/$comp" ]; then
            echo -e "  ${GREEN}✓${NC} $comp - $desc"
        fi
    done
    echo
    echo -e "${CYAN}Quick Start:${NC}"
    echo "  • Run 'hecate status' to check system"
    echo "  • Run 'hecate optimize' to tune performance"
    echo "  • Visit http://localhost:9313 for monitoring dashboard"
    echo
    echo -e "${YELLOW}Note:${NC} Reboot recommended for all optimizations to take effect"
}

# Main installation flow
main() {
    print_header
    check_root
    check_ubuntu
    detect_hardware
    install_binaries
    install_services
    configure_system
    enable_services
    print_summary
}

# Run main function
main "$@"