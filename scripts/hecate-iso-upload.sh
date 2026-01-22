#!/bin/bash
#
# HecateOS ISO Upload Script
# Uploads ISO files to cloud storage using rclone
#
# Supported backends:
#   - Cloudflare R2 (recommended)
#   - AWS S3
#   - Backblaze B2
#   - Any rclone-supported provider
#
# Usage:
#   ./scripts/hecate-iso-upload.sh <iso-file>
#   ./scripts/hecate-iso-upload.sh iso/hecate-os-0.1.0-amd64.iso
#
# Environment variables:
#   HECATE_REMOTE  - rclone remote name (default: HecateOS)
#   HECATE_BUCKET  - bucket/path name (default: releases)
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

# Configuration
REMOTE="${HECATE_REMOTE:-HecateOS}"
BUCKET="${HECATE_BUCKET:-releases}"

# Check arguments
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No ISO file specified${RESET}"
    echo "Usage: $0 <iso-file>"
    echo ""
    echo "Example:"
    echo "  $0 iso/hecate-os-0.1.0-amd64.iso"
    exit 1
fi

ISO_FILE="$1"

# Verify file exists
if [ ! -f "$ISO_FILE" ]; then
    echo -e "${RED}Error: File not found: $ISO_FILE${RESET}"
    exit 1
fi

# Check rclone is installed
if ! command -v rclone &> /dev/null; then
    echo -e "${RED}Error: rclone is not installed${RESET}"
    echo ""
    echo "Install rclone:"
    echo "  curl https://rclone.org/install.sh | sudo bash"
    echo ""
    echo "Or on Ubuntu/Debian:"
    echo "  sudo apt install rclone"
    exit 1
fi

# Check rclone is configured
if ! rclone listremotes 2>/dev/null | grep -q "^${REMOTE}:"; then
    echo -e "${RED}Error: rclone remote '$REMOTE' not configured${RESET}"
    echo ""
    echo "Configure rclone for Cloudflare R2:"
    echo "  rclone config"
    echo ""
    echo "Or set HECATE_REMOTE to use a different remote:"
    echo "  HECATE_REMOTE=MyRemote $0 $ISO_FILE"
    exit 1
fi

# Get file info
ISO_NAME=$(basename "$ISO_FILE")
ISO_SIZE=$(du -h "$ISO_FILE" | cut -f1)

echo -e "${CYAN}╦ ╦┌─┐┌─┐┌─┐┌┬┐┌─┐╔═╗╔═╗${RESET}"
echo -e "${CYAN}╠═╣├┤ │  ├─┤ │ ├┤ ║ ║╚═╗${RESET}"
echo -e "${CYAN}╩ ╩└─┘└─┘┴ ┴ ┴ └─┘╚═╝╚═╝${RESET}"
echo ""
echo -e "${CYAN}ISO Upload${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "File:   ${GREEN}$ISO_NAME${RESET}"
echo -e "Size:   ${GREEN}$ISO_SIZE${RESET}"
echo -e "Remote: ${GREEN}$REMOTE:$BUCKET/${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Generate checksum if not exists
CHECKSUM_FILE="${ISO_FILE}.sha256"
if [ ! -f "$CHECKSUM_FILE" ]; then
    echo -e "${YELLOW}Generating SHA256 checksum...${RESET}"
    sha256sum "$ISO_FILE" > "$CHECKSUM_FILE"
    echo -e "${GREEN}✓ Checksum generated${RESET}"
fi

# Upload ISO
echo -e "${CYAN}Uploading ISO...${RESET}"
rclone copy "$ISO_FILE" "$REMOTE:$BUCKET/" \
    --progress \
    --transfers 4 \
    --checkers 8

echo -e "${GREEN}✓ ISO uploaded${RESET}"

# Upload checksum
echo -e "${CYAN}Uploading checksum...${RESET}"
rclone copy "$CHECKSUM_FILE" "$REMOTE:$BUCKET/" --progress

echo -e "${GREEN}✓ Checksum uploaded${RESET}"

# Upload signature if exists
SIG_FILE="${ISO_FILE}.sig"
if [ -f "$SIG_FILE" ]; then
    echo -e "${CYAN}Uploading signature...${RESET}"
    rclone copy "$SIG_FILE" "$REMOTE:$BUCKET/" --progress
    echo -e "${GREEN}✓ Signature uploaded${RESET}"
fi

# Verify upload
echo ""
echo -e "${CYAN}Verifying upload...${RESET}"
if rclone ls "$REMOTE:$BUCKET/$ISO_NAME" &>/dev/null; then
    echo -e "${GREEN}✓ Upload verified${RESET}"
else
    echo -e "${RED}✗ Upload verification failed${RESET}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Upload complete!${RESET}"
echo ""
echo "Files available at:"
echo "  $REMOTE:$BUCKET/$ISO_NAME"
echo "  $REMOTE:$BUCKET/$(basename $CHECKSUM_FILE)"
if [ -f "$SIG_FILE" ]; then
    echo "  $REMOTE:$BUCKET/$(basename $SIG_FILE)"
fi
echo ""
