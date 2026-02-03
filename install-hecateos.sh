#!/bin/bash
#
# HecateOS Installer Script
# Run this on a fresh Ubuntu 24.04 installation to convert it to HecateOS
#

set -e

echo "==========================================="
echo "  HecateOS Installer"
echo "==========================================="
echo "This will install HecateOS components on Ubuntu 24.04"
echo ""

# Check if running on Ubuntu 24.04
if ! grep -q "Ubuntu 24.04" /etc/os-release 2>/dev/null; then
    echo "Warning: This script is designed for Ubuntu 24.04"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Install dependencies
echo "Installing dependencies..."
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    libssl-dev \
    cpufrequtils \
    tuned \
    irqbalance

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Clone HecateOS repository
echo "Cloning HecateOS repository..."
if [ ! -d "$HOME/hecateos" ]; then
    git clone https://github.com/yourusername/hecate-os.git $HOME/hecateos
fi

# Build HecateOS components
echo "Building HecateOS components..."
cd $HOME/hecateos/rust
cargo build --release

# Install binaries
echo "Installing HecateOS binaries..."
sudo cp target/release/hecate* /usr/local/bin/

# Apply system optimizations
echo "Applying optimizations..."

# Kernel parameters
sudo tee /etc/sysctl.d/99-hecateos.conf << 'EOF'
# HecateOS Performance Optimizations
vm.swappiness = 10
vm.vfs_cache_pressure = 50
vm.dirty_background_ratio = 5
vm.dirty_ratio = 10
net.core.default_qdisc = fq_codel
net.ipv4.tcp_congestion = bbr
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
kernel.sched_autogroup_enabled = 1
EOF

sudo sysctl -p /etc/sysctl.d/99-hecateos.conf

# CPU Governor
echo "Setting CPU governor to performance..."
sudo cpupower frequency-set -g performance 2>/dev/null || \
    echo "performance" | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Create systemd services
echo "Creating systemd services..."

sudo tee /etc/systemd/system/hecated.service << 'EOF'
[Unit]
Description=HecateOS System Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/hecated
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

sudo tee /etc/systemd/system/hecate-monitor.service << 'EOF'
[Unit]
Description=HecateOS Monitoring Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/hecate-monitor
Restart=always
RestartSec=10
Environment="HECATE_MONITOR_PORT=9313"

[Install]
WantedBy=multi-user.target
EOF

# Enable and start services
sudo systemctl daemon-reload
sudo systemctl enable hecated.service
sudo systemctl enable hecate-monitor.service
sudo systemctl start hecated.service
sudo systemctl start hecate-monitor.service

# Update MOTD
sudo tee /etc/motd << 'EOF'

 ██╗  ██╗███████╗ ██████╗ █████╗ ████████╗███████╗ ██████╗ ███████╗
 ██║  ██║██╔════╝██╔════╝██╔══██╗╚══██╔══╝██╔════╝██╔═══██╗██╔════╝
 ███████║█████╗  ██║     ███████║   ██║   █████╗  ██║   ██║███████╗
 ██╔══██║██╔══╝  ██║     ██╔══██║   ██║   ██╔══╝  ██║   ██║╚════██║
 ██║  ██║███████╗╚██████╗██║  ██║   ██║   ███████╗╚██████╔╝███████║
 ╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝ ╚═════╝ ╚══════╝
                                                         
 Performance-Optimized Linux Distribution v0.1.0
 Based on Ubuntu 24.04 LTS
 
 Monitor Dashboard: http://localhost:9313
 
EOF

echo ""
echo "==========================================="
echo "  HecateOS Installation Complete!"
echo "==========================================="
echo ""
echo "Services status:"
systemctl status hecated --no-pager | grep Active
systemctl status hecate-monitor --no-pager | grep Active
echo ""
echo "Dashboard: http://localhost:9313"
echo ""
echo "Run 'hecate-bench sysinfo' to see system information"
echo ""
echo "Reboot recommended to apply all optimizations."
echo "==========================================="