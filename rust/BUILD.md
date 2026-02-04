# HecateOS Build Guide

## Overview

HecateOS is a performance-optimized Linux distribution built on top of Ubuntu. This guide covers building all components and creating a custom ISO.

## Prerequisites

- Rust 1.75+ (for building components)
- Docker (optional, for containerized builds)
- ISO tools: `xorriso` or `genisoimage` (for ISO creation)
- 10GB+ free disk space
- Internet connection (for downloading Ubuntu base ISO)

## Quick Start

```bash
# Build all components
make all

# Create custom ISO (downloads Ubuntu 24.04 automatically)
./target/release/hecate-iso build --download 24.04 -o hecateos.iso --with-binaries

# Or use existing Ubuntu ISO
./target/release/hecate-iso build -i ubuntu-24.04.iso -o hecateos.iso --with-binaries
```

## Component Build Process

### 1. Core Daemon (hecated)

```bash
cd hecated
cargo build --release
```

The system management daemon that handles:
- Resource monitoring
- Performance tuning
- System optimization

### 2. System Monitor (hecate-monitor)

```bash
cd hecate-monitor
cargo build --release
```

Web-based monitoring dashboard (port 9313):
- Real-time metrics
- Performance graphs
- System health status

### 3. Benchmarking Tool (hecate-bench)

```bash
cd hecate-bench
cargo build --release
```

Comprehensive benchmarking suite:
- CPU, memory, disk, network tests
- ML workload benchmarks
- Results comparison

### 4. Package Manager (hecate-pkg)

```bash
cd hecate-pkg
# Set database URL for development
export DATABASE_URL="sqlite:hecate-pkg.db"
cargo build --release
```

Package management system:
- Binary package installation
- Dependency resolution
- Repository management

### 5. GPU Manager (hecate-gpu)

```bash
cd hecate-gpu
cargo build --release
```

GPU optimization and management:
- NVIDIA/AMD detection
- Power management
- Memory optimization

### 6. ML Optimizer (hecate-ml)

```bash
cd hecate-ml
cargo build --release
```

Machine learning workload optimizer:
- Framework detection
- Resource allocation
- Performance profiling

### 7. Development Tools (hecate-dev)

```bash
cd hecate-dev
cargo build --release
```

Development environment tools:
- Performance profiling
- Code optimization
- Build helpers

### 8. Package Signer (hecate-sign)

```bash
cd hecate-sign
cargo build --release
```

Package signing utilities:
- GPG key management
- Package verification
- Repository signing

## ISO Creation

### Using hecate-iso-builder

The `hecate-iso-builder` is our custom tool for creating HecateOS ISOs:

#### Build with automatic download:
```bash
./target/release/hecate-iso build \
    --download 24.04 \
    -o hecateos.iso \
    --with-binaries \
    --config hecate-iso-config.toml
```

#### Build with existing ISO:
```bash
./target/release/hecate-iso build \
    -i ubuntu-24.04-desktop-amd64.iso \
    -o hecateos.iso \
    --with-binaries \
    --with-source
```

#### Configuration

Create a custom configuration with:
```bash
./target/release/hecate-iso init -o my-config.toml
```

Configuration includes:
- Component selection
- Kernel parameters
- System optimizations
- Branding customization

### ISO Builder Commands

- **build** - Create customized ISO
- **extract** - Extract ISO for manual editing
- **repack** - Repack modified ISO
- **init** - Create config template
- **verify** - Verify ISO integrity

## Testing

### In Virtual Machine

```bash
# QEMU
qemu-system-x86_64 -m 4G -cdrom hecateos.iso -enable-kvm

# VirtualBox
VBoxManage createvm --name HecateOS --ostype Ubuntu_64 --register
VBoxManage modifyvm HecateOS --memory 4096 --cpus 2
VBoxManage storagectl HecateOS --name IDE --add ide
VBoxManage storageattach HecateOS --storagectl IDE --port 0 --device 0 --type dvddrive --medium hecateos.iso
VBoxManage startvm HecateOS
```

### On Physical Hardware

```bash
# Write to USB drive (replace sdX with your device)
sudo dd if=hecateos.iso of=/dev/sdX bs=4M status=progress
sync
```

## Build Automation

### Using Make

```bash
# Build everything
make all

# Build specific component
make hecated
make monitor

# Clean build artifacts
make clean

# Run tests
make test

# Build ISO
make iso
```

### Using Docker

```bash
# Build in container
docker build -t hecateos-builder .
docker run -v $(pwd):/build hecateos-builder make all

# Create ISO in container
docker run -v $(pwd):/build hecateos-builder make iso
```

## Local Development Only

This is alpha development software. All builds should be done locally. CI/CD will be added in future releases.

## Performance Optimizations

The ISO builder automatically applies:

### Kernel Parameters
- `mitigations=off` - Disable CPU vulnerability mitigations
- `intel_pstate=enable` - Enable Intel P-state driver
- `transparent_hugepage=always` - Enable transparent huge pages
- `processor.max_cstate=1` - Limit CPU idle states

### System Settings (sysctl)
- Reduced swappiness (10)
- Optimized cache pressure
- BBR congestion control
- Improved dirty page handling

### CPU Governor
- Performance mode by default
- Turbo boost enabled
- Frequency scaling optimized

## Troubleshooting

### ISO Extraction Fails

If automatic extraction fails, use manual mount:
```bash
sudo mkdir /mnt/iso
sudo mount -o loop input.iso /mnt/iso
cp -r /mnt/iso/* /path/to/extract/
sudo umount /mnt/iso
```

### Missing Dependencies

Install required tools:
```bash
# Ubuntu/Debian
sudo apt-get install xorriso genisoimage squashfs-tools

# Fedora/RHEL
sudo dnf install xorriso genisoimage squashfs-tools

# Arch
sudo pacman -S xorriso cdrtools squashfs-tools
```

### Build Errors

```bash
# Clear cargo cache
cargo clean

# Update dependencies
cargo update

# Rebuild with verbose output
cargo build --release --verbose
```

## Development

### Project Structure
```
rust/
├── hecated/          # Core daemon
├── hecate-monitor/   # Web dashboard
├── hecate-bench/     # Benchmarking
├── hecate-pkg/       # Package manager
├── hecate-gpu/       # GPU management
├── hecate-ml/        # ML optimizer
├── hecate-dev/       # Dev tools
├── hecate-sign/      # Package signing
├── hecate-iso-builder/ # ISO creation tool
└── Makefile          # Build automation
```

### Contributing

1. Fork the repository
2. Create feature branch
3. Make changes and test
4. Submit pull request

### Testing Components

```bash
# Run all tests
cargo test --all

# Run specific component tests
cd hecated && cargo test

# Run with coverage
cargo tarpaulin --out Html
```

## Support

- Issues: [GitHub Issues](https://github.com/hecateos/hecateos)
- Documentation: [docs.hecateos.org](https://docs.hecateos.org)
- Community: [Discord](https://discord.gg/hecateos)

## License

MIT License - See LICENSE file for details.