# HecateOS Installation Guide

## Quick Start

### 1. Download HecateOS

Download the ISO from [Releases](https://github.com/Arakiss/hecate-os/releases).

One ISO, automatic hardware detection. No editions to choose.

### 2. Create Bootable USB

#### Linux/macOS:
```bash
# Find your USB device
lsblk  # or diskutil list on macOS

# Write ISO to USB (replace sdX with your device)
sudo dd if=hecate-os-*.iso of=/dev/sdX bs=4M status=progress sync
```

#### Windows:
- Use [Rufus](https://rufus.ie) or [balenaEtcher](https://etcher.io)
- Select ISO and USB drive
- Use DD mode if prompted

#### Multi-boot with Ventoy:
1. Install [Ventoy](https://ventoy.net) on USB
2. Copy ISO to USB drive
3. Boot and select HecateOS from menu

### 3. Installation Process

1. **Boot from USB**
   - Enter BIOS (F2/F12/DEL during boot)
   - Disable Secure Boot (for NVIDIA drivers)
   - Set USB as first boot device

2. **HecateOS Installer**
   - System automatically detects your hardware
   - Choose installation type:
     - **Automatic**: Let HecateOS decide everything
     - **Custom**: Choose your settings

3. **Disk Setup** (for Dual-Boot)
   ```
   HecateOS will NOT touch Windows partitions

   Select your Linux drive (e.g., Samsung 990 PRO)
   Partitioning scheme:
   - /boot/efi : 1GB (UEFI)
   - /boot     : 2GB
   - /         : 200GB (root)
   - /home     : 100GB
   - /var      : Remaining space
   ```

4. **First Boot Setup**
   - Detects hardware and applies optimizations automatically
   - Driver installation (automatic)

## Hardware Detection

HecateOS automatically detects and optimizes for:

### CPUs
- **Intel 13th/12th Gen**: P-core/E-core optimization
- **AMD Ryzen 7000/5000**: Zen 4/3 optimizations
- **Older CPUs**: Standard performance tuning

### GPUs
- **NVIDIA RTX 40 Series**: Driver 570 + CUDA 12.8
- **NVIDIA RTX 30 Series**: Driver 535 + CUDA 12.3
- **AMD RX 7000/6000**: AMDGPU drivers
- **Intel Arc/Xe**: Intel media drivers

### Memory
- **64GB+**: High performance mode, low swappiness
- **32GB+**: Balanced mode
- **<32GB**: Optimized swap settings

## Post-Installation

### HecateOS CLI

The `hecate` command provides system management:

```bash
hecate info          # Show system info and applied optimizations
hecate info --all    # Detailed hardware and optimization info
hecate update        # Update system packages and run migrations
hecate optimize      # Re-detect hardware and apply optimizations
hecate driver        # Show GPU driver status
hecate driver install  # Install GPU drivers
hecate benchmark     # Run performance benchmark
```

### Verify Installation
```bash
# Check system info
hecate info --all

# Run benchmarks
sudo hecate benchmark

# Test GPU
nvidia-smi  # For NVIDIA
glxinfo | grep renderer  # For any GPU

# Test Docker with GPU
docker run --rm --gpus all nvidia/cuda:12.8.0-base-ubuntu24.04 nvidia-smi
```

## Dual-Boot with Windows

HecateOS automatically detects Windows installations:

1. **GRUB Menu**: Shows both HecateOS and Windows
2. **Default Boot**: HecateOS (changeable)
3. **Timeout**: 10 seconds

To change default to Windows:
```bash
sudo nano /etc/default/grub
# Set GRUB_DEFAULT=2 (or the Windows entry number)
sudo update-grub
```

## Troubleshooting

### NVIDIA Driver Issues
```bash
# Reinstall driver
sudo hecate-driver-installer

# Check driver status
nvidia-smi

# If black screen after install
# Boot to recovery mode and run:
sudo apt purge nvidia-*
sudo hecate-driver-installer
```

### Performance Issues
```bash
# Re-detect hardware and reapply optimizations
sudo hecate optimize

# Check current profile
cat /etc/hecate/hardware-profile.json

# View applied optimizations
hecate info --all
```

### Boot Issues
```bash
# Rebuild GRUB
sudo update-grub
sudo grub-install /dev/nvme2n1  # Your Linux drive

# Fix EFI boot
sudo efibootmgr -v  # List entries
sudo efibootmgr -c -d /dev/nvme2n1 -p 1 -L "HecateOS" -l '\EFI\ubuntu\grubx64.efi'
```

## Uninstallation

To remove HecateOS (keeping Windows):

1. Boot into Windows
2. Use Disk Management to delete Linux partitions
3. Run in Admin CMD:
   ```cmd
   bcdedit /set {bootmgr} path \EFI\Microsoft\Boot\bootmgfw.efi
   ```

## System Requirements

### Minimum
- CPU: 4 cores / 8 threads
- RAM: 8GB
- Storage: 128GB SSD
- GPU: Optional (integrated OK)

### Recommended
- CPU: Intel i7-12700K / AMD Ryzen 7 5800X or better
- RAM: 32GB DDR5/DDR4
- Storage: 512GB NVMe SSD
- GPU: NVIDIA RTX 3060 or better

## Support

- **GitHub Issues**: https://github.com/Arakiss/hecate-os/issues
