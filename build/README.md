# HecateOS Build System

This directory contains the build scripts for creating a HecateOS ISO image based on Ubuntu 24.04 LTS.

## Overview

HecateOS is built on top of Ubuntu 24.04 LTS (Noble Numbat) with the following additions:
- Custom performance optimizations
- HecateOS system components (written in Rust)
- Optimized kernel parameters
- Performance-focused default configuration

## Build Requirements

- Ubuntu 22.04+ host system (for building)
- At least 10GB free disk space
- 4GB+ RAM
- sudo privileges
- Internet connection (for downloading packages)

## Quick Start

1. **Build the Rust components** (if not already built):
   ```bash
   cd ../rust
   cargo build --release
   ```

2. **Build the ISO**:
   ```bash
   cd build
   chmod +x build-iso.sh
   sudo ./build-iso.sh
   ```
   This will create `output/hecateos-0.1.0-amd64.iso`

3. **Test in a VM**:
   ```bash
   chmod +x test-vm.sh
   ./test-vm.sh
   ```

## Build Scripts

### `build-iso.sh`
Main build script that:
- Creates Ubuntu 24.04 base system using debootstrap
- Installs HecateOS Rust components
- Applies performance optimizations
- Creates bootable ISO image

### `test-vm.sh`
Quick VM testing script that:
- Launches HecateOS ISO in QEMU
- Configures networking (forwards port 9313 for monitoring dashboard)
- Provides 4GB RAM and 4 CPUs by default

### Alternative: `build-kernel.sh` (Optional)
Builds a custom optimized kernel from source (not required for Ubuntu-based build)

### Alternative: `create-rootfs.sh` (Optional)
Creates a minimal rootfs from scratch (not required for Ubuntu-based build)

## ISO Contents

The generated ISO includes:

### Base System
- Ubuntu 24.04 LTS minimal installation
- Latest HWE kernel (Hardware Enablement)
- Essential system utilities

### HecateOS Components
- **hecated** - System daemon for optimization
- **hecate-monitor** - Real-time monitoring server (port 9313)
- **hecate-bench** - Performance benchmarking tool
- **hecate-pkg** - Package management system
- **hecate-dev** - Development tools
- **hecate-lint** - Code quality checker
- **hecate-sign** - Digital signature tool

### Performance Optimizations
- BBR TCP congestion control
- Transparent huge pages enabled
- CPU governor set to performance
- Optimized sysctl parameters
- I/O scheduler tuning

## Testing

### In QEMU/KVM
```bash
./test-vm.sh
```

### In VirtualBox
1. Create new VM with:
   - Type: Linux
   - Version: Ubuntu 64-bit
   - Memory: 4096 MB+
   - Create virtual hard disk (20GB+)
2. Settings:
   - System → Enable EFI
   - System → Processors: 4+
   - Storage → Add ISO to optical drive
   - Network → NAT or Bridged
3. Start VM

### In VMware
1. Create new VM
2. Select "Linux" → "Ubuntu 64-bit"
3. Use ISO as installation media
4. Allocate 4GB+ RAM, 20GB+ disk
5. Start VM

## Default Credentials

- Username: `hecateos`
- Password: `hecateos`
- Root password: `hecateos`

## Services

After boot, these services are available:

- **System Daemon**: `systemctl status hecated`
- **Monitoring Server**: `systemctl status hecate-monitor`
- **Web Dashboard**: http://localhost:9313

## Directory Structure

```
build/
├── build-iso.sh          # Main ISO builder (Ubuntu-based)
├── test-vm.sh           # QEMU test script
├── build-kernel.sh      # Optional: Custom kernel build
├── create-rootfs.sh     # Optional: Minimal rootfs
├── iso-build/           # Working directory (created during build)
│   ├── squashfs/       # Ubuntu root filesystem
│   └── cd/             # ISO contents
└── output/             # Build outputs
    └── hecateos-0.1.0-amd64.iso
```

## Customization

To customize the build, edit `build-iso.sh`:

- **Packages**: Add to the `apt-get install` section
- **Services**: Add systemd service files
- **Optimizations**: Modify sysctl.d configuration
- **Branding**: Update MOTD and boot messages

## Troubleshooting

### Build Failures
- Ensure you have sudo privileges
- Check internet connection
- Verify at least 10GB free space
- Run with `sudo bash -x build-iso.sh` for debug output

### VM Won't Boot
- Enable virtualization in BIOS
- For KVM: Check if `/dev/kvm` exists
- Try without `-enable-kvm` flag
- Increase RAM allocation

### Services Not Starting
- Check logs: `journalctl -u hecated`
- Verify binaries exist: `ls /opt/hecateos/bin/`
- Check permissions: `ls -la /opt/hecateos/bin/`

## Next Steps

After successful build and test:

1. **Performance Testing**:
   ```bash
   hecate-bench all
   ```

2. **System Info**:
   ```bash
   hecate-bench sysinfo
   ```

3. **Monitor Dashboard**:
   Open http://localhost:9313 in browser

4. **Package Management**:
   ```bash
   hecate-pkg list
   hecate-pkg search <package>
   ```

## Contributing

To add new features:

1. Implement in Rust (`../rust/`)
2. Build: `cargo build --release`
3. Update `build-iso.sh` to include new binaries
4. Test in VM before release

## License

MIT License - See LICENSE file in repository root