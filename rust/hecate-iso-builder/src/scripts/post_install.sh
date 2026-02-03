#!/bin/bash
# HecateOS Post-Installation Script

echo "Running HecateOS post-installation tasks..."

# Enable services
systemctl enable hecated
systemctl enable hecate-monitor

# Start services
systemctl start hecated
systemctl start hecate-monitor

echo "Post-installation complete!"