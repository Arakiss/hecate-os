# HecateOS Installation Guide 🚀

## Quick Start

### 1. Download HecateOS

Choose your edition based on your hardware:

- **Ultimate**: RTX 4080/4090 + 64GB+ RAM → AI/ML powerhouse
- **Workstation**: RTX 3060+ + 32GB+ RAM → Professional development
- **Gaming**: Any NVIDIA GPU + 16GB+ RAM → Gaming optimized
- **Developer**: 16GB+ RAM → Coding focused
- **Lite**: 8GB+ RAM → Standard computing
- **Server**: Headless servers → No GUI

### 2. Create Bootable USB

#### Linux/macOS:
```bash
# Find your USB device (be careful!)
lsblk  # or diskutil list on macOS

# Write ISO to USB (replace sdX with your device)
sudo dd if=hecate-os-ultimate-*.iso of=/dev/sdX bs=4M status=progress sync
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
   - System will automatically detect your hardware
   - Recommended edition will be highlighted
   - Choose installation type:
     - **Automatic**: Let HecateOS decide everything
     - **Custom**: Choose your settings

3. **Disk Setup** (for Dual-Boot)
   ```
   ⚠️ IMPORTANT: HecateOS will NOT touch Windows partitions
   
   Select your Linux drive (e.g., Samsung 990 PRO)
   Partitioning scheme:
   - /boot/efi : 1GB (UEFI)
   - /boot     : 2GB 
   - /         : 200GB (root)
   - /home     : 100GB
   - /var      : Remaining space
   ```

4. **First Boot Setup**
   - Welcome wizard launches automatically
   - Detects hardware and applies optimizations
   - Select your profile and preferences
   - Driver installation (automatic)

## Hardware Detection & Optimization

HecateOS automatically detects and optimizes for:

### CPUs
- **Intel 13th/12th Gen**: P-core/E-core optimization
- **AMD Ryzen 7000/5000**: Zen 4/3 optimizations
- **Older CPUs**: Standard performance tuning

### GPUs
- **NVIDIA RTX 40 Series**: Driver 550 + CUDA 12.6
- **NVIDIA RTX 30 Series**: Driver 535 + CUDA 12.3
- **AMD RX 7000/6000**: AMDGPU/PRO drivers
- **Intel Arc/Xe**: Intel media drivers

### Memory
- **128GB+**: Extreme performance mode
- **64GB+**: High performance mode
- **32GB+**: Balanced mode
- **<32GB**: Optimized swap settings

## Post-Installation

### Run Post-Install Script
```bash
sudo hecate-post-install
```

This will:
- Update all packages
- Configure NVIDIA Container Toolkit
- Apply final optimizations
- Detect Windows for dual-boot

### Verify Installation
```bash
# Check system info
hecate-info --all

# Run benchmarks
sudo hecate-benchmark

# Test GPU
nvidia-smi  # For NVIDIA
glxinfo | grep renderer  # For any GPU

# Test Docker with GPU
docker run --rm --gpus all nvidia/cuda:12.6.0-base-ubuntu24.04 nvidia-smi
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
# Re-detect hardware
sudo /usr/local/bin/hecate-hardware-detect

# Reapply optimizations
sudo /usr/local/bin/hecate-apply-optimizations

# Check current profile
cat /etc/hecate/hardware-profile.json
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

## Switching Editions

Change your HecateOS edition after installation:

```bash
# Check current edition
hecate-info --version

# Switch edition (downloads packages)
sudo hecate-edition switch ultimate

# Available: ultimate, workstation, gaming, developer, lite, server
```

## Network Installation

Install HecateOS over network (PXE):

```bash
# On server
sudo hecate-pxe-server start

# Boot target machine from network
# Select HecateOS from PXE menu
```

## Uninstallation

To remove HecateOS (keeping Windows):

1. Boot into Windows
2. Use Disk Management to delete Linux partitions
3. Run in Admin CMD:
   ```cmd
   bcdedit /set {bootmgr} path \EFI\Microsoft\Boot\bootmgfw.efi
   ```

## Support

- **Documentation**: https://hecateos.dev/docs
- **Discord**: https://discord.gg/hecate-os
- **GitHub Issues**: https://github.com/hecate-os/hecate-os/issues

## System Requirements

### Minimum
- CPU: 4 cores / 8 threads
- RAM: 8GB
- Storage: 128GB SSD
- GPU: Optional (integrated OK for Lite/Developer)

### Recommended
- CPU: Intel i7-12700K / AMD Ryzen 7 5800X or better
- RAM: 32GB DDR5/DDR4
- Storage: 512GB NVMe SSD
- GPU: NVIDIA RTX 3060 or better

### Optimal (for Ultimate)
- CPU: Intel i9-13900K / AMD Ryzen 9 7950X
- RAM: 128GB DDR5-6000+
- Storage: 2TB NVMe Gen5
- GPU: NVIDIA RTX 4090

---

**Welcome to HecateOS - Unleash the beast within your machine!** 🔥