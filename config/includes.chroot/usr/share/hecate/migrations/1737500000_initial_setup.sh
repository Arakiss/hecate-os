#!/bin/bash
#
# HecateOS Migration: Initial Setup
# Creates required directories and configuration files
#

set -e

echo "Setting up HecateOS directories..."

# Create config directory
mkdir -p /etc/hecate

# Write version file
echo "0.1.0" > /etc/hecate/version

# Create empty profile if not exists
if [ ! -f /etc/hecate/hardware-profile.json ]; then
    echo '{"system":{"profile":"standard","version":"0.1.0"}}' > /etc/hecate/hardware-profile.json
fi

# Create user config directory
mkdir -p /etc/hecate/conf.d

echo "Initial setup complete"
