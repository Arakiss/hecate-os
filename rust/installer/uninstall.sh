#!/bin/bash
#
# HecateOS Uninstaller Script
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Installation directories
INSTALL_PREFIX="/usr/local"
BIN_DIR="${INSTALL_PREFIX}/bin"
LIB_DIR="${INSTALL_PREFIX}/lib/hecate"
CONFIG_DIR="/etc/hecate"
SYSTEMD_DIR="/etc/systemd/system"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root${NC}"
    echo "Please run: sudo $0"
    exit 1
fi

echo -e "${CYAN}╔════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║      HecateOS Uninstallation Script        ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════╝${NC}"
echo

read -p "Are you sure you want to uninstall HecateOS? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstallation cancelled."
    exit 0
fi

# Stop and disable services
echo -e "\n${YELLOW}Stopping services...${NC}"
systemctl stop hecated.service 2>/dev/null || true
systemctl stop hecate-monitor.service 2>/dev/null || true
systemctl disable hecated.service 2>/dev/null || true
systemctl disable hecate-monitor.service 2>/dev/null || true

# Remove service files
echo -e "${YELLOW}Removing service files...${NC}"
rm -f "$SYSTEMD_DIR/hecated.service"
rm -f "$SYSTEMD_DIR/hecate-monitor.service"
systemctl daemon-reload

# Remove binaries
echo -e "${YELLOW}Removing binaries...${NC}"
rm -f "$BIN_DIR"/hecate*

# Remove libraries and configs
echo -e "${YELLOW}Removing configuration...${NC}"
rm -rf "$LIB_DIR"

read -p "Remove configuration files? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$CONFIG_DIR"
fi

echo -e "\n${GREEN}HecateOS has been uninstalled.${NC}"