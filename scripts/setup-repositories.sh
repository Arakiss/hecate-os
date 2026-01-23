#!/bin/bash
#
# HecateOS Repository Setup Script
# Adds official third-party repositories for latest software
#

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
RESET='\033[0m'

echo -e "${CYAN}HecateOS Repository Setup${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create keyrings directory
sudo mkdir -p /usr/share/keyrings

# NVIDIA CUDA Repository (January 2026)
echo -e "${YELLOW}Adding NVIDIA CUDA Repository...${RESET}"
wget -qO- https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb | sudo dpkg -i - 2>/dev/null || true
echo "deb [signed-by=/usr/share/keyrings/cuda-archive-keyring.gpg] https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/ /" | \
    sudo tee /etc/apt/sources.list.d/cuda.list

# Docker CE Repository
echo -e "${YELLOW}Adding Docker CE Repository...${RESET}"
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu noble stable" | \
    sudo tee /etc/apt/sources.list.d/docker.list

# GitHub CLI Repository
echo -e "${YELLOW}Adding GitHub CLI Repository...${RESET}"
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | \
    sudo tee /etc/apt/sources.list.d/github-cli.list

# PostgreSQL APT Repository
echo -e "${YELLOW}Adding PostgreSQL APT Repository...${RESET}"
curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo gpg --dearmor -o /usr/share/keyrings/postgresql-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/postgresql-archive-keyring.gpg] https://apt.postgresql.org/pub/repos/apt noble-pgdg main" | \
    sudo tee /etc/apt/sources.list.d/postgresql.list

# NodeSource Repository for Node.js 22.x
echo -e "${YELLOW}Adding NodeSource Repository (Node.js 22.x)...${RESET}"
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -

# HecateOS Official Repository (when available)
# echo -e "${YELLOW}Adding HecateOS Official Repository...${RESET}"
# curl -fsSL https://repo.hecate-os.io/KEY.gpg | sudo gpg --dearmor -o /usr/share/keyrings/hecate-archive-keyring.gpg
# echo "deb [signed-by=/usr/share/keyrings/hecate-archive-keyring.gpg] https://repo.hecate-os.io/ubuntu noble main" | \
#     sudo tee /etc/apt/sources.list.d/hecate.list

# Update package lists
echo -e "${CYAN}Updating package lists...${RESET}"
sudo apt update

echo -e "${GREEN}✓ Repository setup complete!${RESET}"
echo ""
echo "Available repositories:"
echo "  - NVIDIA CUDA (cuda-toolkit, cudnn, tensorrt)"
echo "  - Docker CE (docker-ce, docker-compose-plugin)"
echo "  - GitHub CLI (gh)"
echo "  - PostgreSQL (postgresql-16, pgadmin4)"
echo "  - NodeSource (nodejs 22.x LTS)"
echo ""
echo "Install latest packages with:"
echo "  sudo apt install cuda-12-6 docker-ce gh postgresql-16 nodejs"