# HecateOS ISO Builder

A professional tool for customizing Ubuntu ISOs with HecateOS components.

## ✨ Features

- **Automatic extraction** of Ubuntu ISOs
- **Component injection** for HecateOS (binaries, configuration, scripts)
- **Complete customization** of boot, branding and optimizations
- **Automatic repackaging** with UEFI/Legacy support
- **TOML configuration** for reproducible builds

## 📦 Installation

```bash
cd rust
cargo build --release
sudo cp target/release/hecate-iso /usr/local/bin/
```

## 🚀 Quick Start

### 1. Download Ubuntu ISO
```bash
wget https://releases.ubuntu.com/24.04/ubuntu-24.04-live-server-amd64.iso
```

### 2. Create customized ISO
```bash
# Simple build with binaries
hecate-iso build -i ubuntu-24.04-live-server-amd64.iso -o hecateos.iso --with-binaries

# Complete build with source
hecate-iso build -i ubuntu-24.04-live-server-amd64.iso -o hecateos.iso --with-binaries --with-source
```

### 3. Use custom configuration
```bash
# Generate configuration template
hecate-iso init -o my-config.toml

# Edit my-config.toml as needed

# Build with configuration
hecate-iso build -i ubuntu.iso -o hecateos.iso -c my-config.toml
```

## 🔧 Commands

### `build`
Build a customized HecateOS ISO.

```bash
hecate-iso build [OPTIONS] --input <INPUT>

OPTIONS:
    -i, --input <INPUT>           Input Ubuntu ISO
    -o, --output <OUTPUT>         Output ISO [default: hecateos.iso]
    -c, --config <CONFIG>         TOML configuration file
        --with-binaries           Include compiled binaries
        --with-source             Include source code
```

### `extract`
Extract an ISO for manual modification.

```bash
hecate-iso extract <ISO> [OPTIONS]

OPTIONS:
    -o, --output <OUTPUT>         Output directory [default: ./iso-extract]
```

### `repack`
Repackage a modified directory into an ISO.

```bash
hecate-iso repack <DIR> [OPTIONS]

OPTIONS:
    -o, --output <OUTPUT>         Output ISO file [default: custom.iso]
    -l, --label <LABEL>           Volume label [default: HECATEOS]
```

### `init`
Create a configuration template.

```bash
hecate-iso init [OPTIONS]

OPTIONS:
    -o, --output <OUTPUT>         Configuration file [default: hecate-iso.toml]
```

### `verify`
Verify that an ISO contains HecateOS components.

```bash
hecate-iso verify <ISO>
```

## 📝 Configuration

The TOML configuration file allows customization of all aspects:

```toml
[metadata]
name = "HecateOS"
version = "0.1.0"
base_distro = "Ubuntu 24.04 LTS"
architecture = "amd64"

[components]
include_binaries = ["hecated", "hecate-monitor", "hecate-bench"]
include_source = false
include_docs = true
additional_packages = ["build-essential", "cpufrequtils"]

[optimizations]
kernel_params = ["mitigations=off", "transparent_hugepage=always"]
cpu_governor = "performance"
io_scheduler = "bfq"

[optimizations.sysctl_settings]
"vm.swappiness" = "10"
"net.ipv4.tcp_congestion" = "bbr"

[branding]
distro_name = "HecateOS"
distro_version = "0.1.0 Performance Beast"
motd = """
 ██╗  ██╗███████╗ ██████╗ █████╗ ████████╗███████╗
 ██║  ██║██╔════╝██╔════╝██╔══██╗╚══██╔══╝██╔════╝
 ███████║█████╗  ██║     ███████║   ██║   █████╗  
 ██╔══██║██╔══╝  ██║     ██╔══██║   ██║   ██╔══╝  
 ██║  ██║███████╗╚██████╗██║  ██║   ██║   ███████╗
 ╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝
"""
```

## 🏗️ Build Process

1. **Extraction**: Decompresses the Ubuntu ISO
2. **Binary injection**: Copies compiled HecateOS binaries
3. **Configuration**: Adds configuration files and scripts
4. **Customization**: Modifies boot menu, MOTD, etc.
5. **Repackaging**: Creates new bootable ISO

## 📂 ISO Structure

```
hecateos.iso
├── hecateos/
│   ├── bin/               # HecateOS binaries
│   │   ├── hecated
│   │   ├── hecate-monitor
│   │   └── ...
│   ├── config/            # Configuration
│   │   ├── hecate.toml
│   │   ├── 99-hecateos.conf
│   │   └── systemd/
│   ├── source/            # Source code (optional)
│   └── install.sh         # Installation script
├── boot/                  # Modified boot configuration
└── [Ubuntu base files]
```

## 🔍 Verification

To verify that an ISO has HecateOS components:

```bash
hecate-iso verify hecateos.iso

# Output:
ISO Contents:
  ✅ HecateOS directory found
  ✅ Binaries directory found:
      hecated (2.1 MB)
      hecate-monitor (3.5 MB)
  ✅ Install script found
  ✅ Configuration directory found
  ✅ Boot menu customized for HecateOS
```

## 🐛 Troubleshooting

### "No ISO creation tool found"
Install required tools:
```bash
sudo apt-get install xorriso genisoimage
```

### "Release binaries not found"
Build binaries first:
```bash
cd rust
cargo build --release
```

### "Failed to extract ISO"
Install extraction tools:
```bash
sudo apt-get install p7zip-full bsdtar
```

## 🚀 Complete Example

```bash
# 1. Prepare environment
cd rust
cargo build --release

# 2. Download Ubuntu
wget https://releases.ubuntu.com/24.04/ubuntu-24.04-live-server-amd64.iso

# 3. Create custom configuration
./target/release/hecate-iso init -o production.toml
# [Edit production.toml]

# 4. Build ISO
./target/release/hecate-iso build \
    -i ubuntu-24.04-live-server-amd64.iso \
    -o hecateos-production.iso \
    -c production.toml \
    --with-binaries

# 5. Verify
./target/release/hecate-iso verify hecateos-production.iso

# 6. Test in VM
qemu-system-x86_64 -m 4G -cdrom hecateos-production.iso
```

## 📄 License

MIT - See LICENSE file