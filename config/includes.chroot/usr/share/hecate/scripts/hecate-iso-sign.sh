#!/bin/bash
#
# HecateOS ISO Signing Script
# Signs ISO files with GPG for verification
#
# Usage:
#   ./scripts/hecate-iso-sign.sh <iso-file> [gpg-key-id]
#
# Environment variables:
#   HECATE_GPG_KEY - GPG key ID to use for signing
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

# Check arguments
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No ISO file specified${RESET}"
    echo "Usage: $0 <iso-file> [gpg-key-id]"
    exit 1
fi

ISO_FILE="$1"
GPG_KEY="${2:-${HECATE_GPG_KEY:-}}"

# Verify file exists
if [ ! -f "$ISO_FILE" ]; then
    echo -e "${RED}Error: File not found: $ISO_FILE${RESET}"
    exit 1
fi

# Check gpg is installed
if ! command -v gpg &> /dev/null; then
    echo -e "${RED}Error: gpg is not installed${RESET}"
    echo "Install gnupg:"
    echo "  sudo apt install gnupg"
    exit 1
fi

# Get file info
ISO_NAME=$(basename "$ISO_FILE")
SIG_FILE="${ISO_FILE}.sig"

echo -e "${CYAN}Signing: $ISO_NAME${RESET}"

# Sign the file
if [ -n "$GPG_KEY" ]; then
    echo -e "Using key: ${GREEN}$GPG_KEY${RESET}"
    gpg --armor --detach-sign --local-user "$GPG_KEY" --output "$SIG_FILE" "$ISO_FILE"
else
    echo -e "${YELLOW}No key specified, using default GPG key${RESET}"
    gpg --armor --detach-sign --output "$SIG_FILE" "$ISO_FILE"
fi

if [ -f "$SIG_FILE" ]; then
    echo -e "${GREEN}✓ Signature created: $(basename $SIG_FILE)${RESET}"

    # Verify the signature
    echo -e "${CYAN}Verifying signature...${RESET}"
    if gpg --verify "$SIG_FILE" "$ISO_FILE" 2>/dev/null; then
        echo -e "${GREEN}✓ Signature verified${RESET}"
    else
        echo -e "${RED}✗ Signature verification failed${RESET}"
        exit 1
    fi
else
    echo -e "${RED}✗ Failed to create signature${RESET}"
    exit 1
fi
