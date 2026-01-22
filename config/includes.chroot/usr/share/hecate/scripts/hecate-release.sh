#!/bin/bash
#
# HecateOS Release Script
# Full release workflow: Build → Checksum → Sign → Upload → GitHub Release
#
# Usage:
#   ./scripts/hecate-release.sh [version]
#
# Example:
#   ./scripts/hecate-release.sh 0.1.0
#   ./scripts/hecate-release.sh  # Uses version from VERSION file
#
# Environment variables:
#   HECATE_SKIP_BUILD   - Skip build step (use existing ISO)
#   HECATE_SKIP_SIGN    - Skip GPG signing
#   HECATE_SKIP_UPLOAD  - Skip cloud upload
#   HECATE_GPG_KEY      - GPG key ID for signing
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
RESET='\033[0m'

# Read version
if [ -n "$1" ]; then
    VERSION="$1"
elif [ -f "$PROJECT_DIR/VERSION" ]; then
    VERSION=$(cat "$PROJECT_DIR/VERSION" | tr -d '[:space:]')
else
    VERSION="0.1.0"
fi

# ISO path
ISO_DIR="$PROJECT_DIR/iso"
ISO_NAME="hecate-os-${VERSION}-amd64"
ISO_FILE="$ISO_DIR/${ISO_NAME}.iso"

echo -e "${PURPLE}"
echo "╦ ╦┌─┐┌─┐┌─┐┌┬┐┌─┐╔═╗╔═╗"
echo "╠═╣├┤ │  ├─┤ │ ├┤ ║ ║╚═╗"
echo "╩ ╩└─┘└─┘┴ ┴ ┴ └─┘╚═╝╚═╝"
echo -e "${RESET}"
echo -e "${CYAN}Release v${VERSION}${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Build ISO
if [ -z "${HECATE_SKIP_BUILD:-}" ]; then
    echo -e "${CYAN}[1/5] Building ISO...${RESET}"
    cd "$PROJECT_DIR"

    if [ -f "docker-compose.yml" ]; then
        docker compose run --rm builder
    else
        sudo ./build.sh build
    fi

    # Find the latest ISO
    ISO_FILE=$(ls -t "$ISO_DIR"/*.iso 2>/dev/null | head -1)
    if [ -z "$ISO_FILE" ]; then
        echo -e "${RED}Error: No ISO file found after build${RESET}"
        exit 1
    fi
    echo -e "${GREEN}✓ Build complete: $(basename $ISO_FILE)${RESET}"
else
    echo -e "${YELLOW}[1/5] Skipping build (HECATE_SKIP_BUILD set)${RESET}"
    if [ ! -f "$ISO_FILE" ]; then
        # Try to find any ISO
        ISO_FILE=$(ls -t "$ISO_DIR"/*.iso 2>/dev/null | head -1)
        if [ -z "$ISO_FILE" ]; then
            echo -e "${RED}Error: No ISO file found${RESET}"
            exit 1
        fi
    fi
fi

echo ""

# Step 2: Generate checksums
echo -e "${CYAN}[2/5] Generating checksums...${RESET}"
cd "$(dirname "$ISO_FILE")"
ISO_BASENAME=$(basename "$ISO_FILE")

sha256sum "$ISO_BASENAME" > "${ISO_BASENAME}.sha256"
md5sum "$ISO_BASENAME" > "${ISO_BASENAME}.md5"

echo -e "${GREEN}✓ Checksums generated${RESET}"
echo "  SHA256: $(cat ${ISO_BASENAME}.sha256 | cut -d' ' -f1 | head -c 16)..."
echo ""

# Step 3: Sign ISO
if [ -z "${HECATE_SKIP_SIGN:-}" ]; then
    echo -e "${CYAN}[3/5] Signing ISO...${RESET}"
    if command -v gpg &> /dev/null && gpg --list-secret-keys 2>/dev/null | grep -q sec; then
        "$SCRIPT_DIR/hecate-iso-sign.sh" "$ISO_FILE"
        echo -e "${GREEN}✓ ISO signed${RESET}"
    else
        echo -e "${YELLOW}⚠ No GPG keys available, skipping signing${RESET}"
    fi
else
    echo -e "${YELLOW}[3/5] Skipping signing (HECATE_SKIP_SIGN set)${RESET}"
fi

echo ""

# Step 4: Upload to cloud storage
if [ -z "${HECATE_SKIP_UPLOAD:-}" ]; then
    echo -e "${CYAN}[4/5] Uploading to cloud storage...${RESET}"
    if command -v rclone &> /dev/null && rclone listremotes 2>/dev/null | grep -q .; then
        "$SCRIPT_DIR/hecate-iso-upload.sh" "$ISO_FILE"
        echo -e "${GREEN}✓ Uploaded to cloud storage${RESET}"
    else
        echo -e "${YELLOW}⚠ rclone not configured, skipping upload${RESET}"
    fi
else
    echo -e "${YELLOW}[4/5] Skipping upload (HECATE_SKIP_UPLOAD set)${RESET}"
fi

echo ""

# Step 5: Create GitHub Release
echo -e "${CYAN}[5/5] Creating GitHub release...${RESET}"
if command -v gh &> /dev/null; then
    cd "$PROJECT_DIR"

    # Check if tag exists
    if git tag -l "v${VERSION}" | grep -q .; then
        echo -e "${YELLOW}Tag v${VERSION} already exists${RESET}"
    else
        git tag -a "v${VERSION}" -m "Release v${VERSION}"
        git push origin "v${VERSION}"
        echo -e "${GREEN}✓ Tag v${VERSION} created and pushed${RESET}"
    fi

    # Create release
    RELEASE_NOTES="## HecateOS v${VERSION}

### Downloads

**ISO Image:**
- Direct: [hecate-os-${VERSION}-amd64.iso](https://github.com/Arakiss/hecate-os/releases/download/v${VERSION}/hecate-os-${VERSION}-amd64.iso)

**Checksums:**
- SHA256: \`$(cat "$ISO_DIR/${ISO_BASENAME}.sha256" | cut -d' ' -f1)\`

### Changes

See [CHANGELOG.md](https://github.com/Arakiss/hecate-os/blob/main/CHANGELOG.md) for details.

### Installation

1. Download the ISO
2. Verify checksum: \`sha256sum -c hecate-os-${VERSION}-amd64.iso.sha256\`
3. Write to USB: \`sudo dd if=hecate-os-${VERSION}-amd64.iso of=/dev/sdX bs=4M status=progress\`
4. Boot and install
"

    if gh release view "v${VERSION}" &>/dev/null; then
        echo -e "${YELLOW}Release v${VERSION} already exists, updating...${RESET}"
        gh release upload "v${VERSION}" \
            "$ISO_FILE" \
            "${ISO_FILE}.sha256" \
            --clobber
    else
        gh release create "v${VERSION}" \
            "$ISO_FILE" \
            "${ISO_FILE}.sha256" \
            --title "HecateOS v${VERSION}" \
            --notes "$RELEASE_NOTES"
    fi

    echo -e "${GREEN}✓ GitHub release created${RESET}"
else
    echo -e "${YELLOW}⚠ gh CLI not available, skipping GitHub release${RESET}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Release v${VERSION} complete!${RESET}"
echo ""
echo "Artifacts:"
echo "  ISO:      $ISO_FILE"
echo "  SHA256:   ${ISO_FILE}.sha256"
[ -f "${ISO_FILE}.sig" ] && echo "  Signature: ${ISO_FILE}.sig"
echo ""
