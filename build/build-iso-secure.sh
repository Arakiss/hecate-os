#!/bin/bash
# HecateOS ISO Build Script - SECURE VERSION
# This script builds the HecateOS ISO without hardcoded credentials

set -e

# Security: Use environment variables or prompt for passwords
if [ -z "$HECATE_PASSWORD" ]; then
    echo "Please set HECATE_PASSWORD environment variable"
    echo "Example: export HECATE_PASSWORD='your-secure-password'"
    echo ""
    echo "Or use a random password generator:"
    echo "export HECATE_PASSWORD=\$(openssl rand -base64 32)"
    exit 1
fi

if [ -z "$ROOT_PASSWORD" ]; then
    ROOT_PASSWORD="$HECATE_PASSWORD"
fi

echo "Building HecateOS ISO with secure credentials..."
echo "Note: Passwords are provided via environment variables"

# Create users with secure passwords from environment
echo "Creating users with secure passwords..."
# echo "hecateos:\$HECATE_PASSWORD" | chpasswd
# echo "root:\$ROOT_PASSWORD" | chpasswd

echo "Build script template ready. Implement actual build logic here."
echo "Remember: NEVER hardcode passwords in scripts!"