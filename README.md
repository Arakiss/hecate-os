# HecateOS

## The Performance-Optimized Linux Distribution

HecateOS is a **Linux distribution based on Ubuntu 24.04 LTS** that automatically optimizes your system for maximum performance. Built specifically for gaming, machine learning, and high-performance computing workloads.

> ⚠️ **ALPHA** - Early development release. Expect rapid changes and improvements.

## Why HecateOS?

**Your hardware deserves better.** HecateOS automatically detects and optimizes for your specific hardware configuration - no manual tweaking required. Get the stability of Ubuntu with performance that rivals custom-tuned systems.

### 🚀 Key Features

- **Automatic Performance Tuning** - Detects your hardware and applies optimal settings on first boot
- **Gaming Ready** - Pre-configured GPU drivers, kernel optimizations, and low-latency settings
- **ML/AI Optimized** - Automatic PyTorch/TensorFlow tuning, CUDA configuration, and batch size optimization
- **Real-time Monitoring** - Beautiful web dashboard showing system performance and thermals
- **Lightweight** - System services use < 50MB RAM thanks to Rust-based components
- **Ubuntu Compatible** - Full compatibility with Ubuntu packages and software

## Quick Start

### Download ISO

```bash
# Coming soon - Pre-built ISOs will be available
# For now, build from source (see below)
```

### Build from Source

```bash
# Prerequisites
sudo apt-get update
sudo apt-get install -y p7zip-full git build-essential curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build HecateOS
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os/rust
cargo build --release --workspace
./target/release/hecate-dev iso create --download 24.04 --output hecateos.iso
```

### Installation

1. **Write to USB**: `sudo dd if=hecateos.iso of=/dev/sdX bs=4M status=progress`
2. **Boot from USB** and follow the installer
3. **Enjoy** your automatically optimized system

## What's Different?

### 🎮 Gaming Performance
- **GPU Auto-Configuration** - NVIDIA and AMD drivers pre-configured and optimized
- **Low Latency Kernel** - Custom kernel parameters for minimal input lag
- **Game Mode** - Automatic CPU governor and priority adjustments when gaming

### 🤖 Machine Learning
- **CUDA/ROCm Ready** - Automatic detection and configuration
- **Framework Optimization** - PyTorch and TensorFlow auto-tuned for your hardware
- **Distributed Training** - Built-in support for multi-GPU setups

### 💻 System Performance
- **Smart CPU Scheduling** - Automatic thread pinning and NUMA optimization
- **Memory Management** - Transparent huge pages and swappiness tuning
- **I/O Optimization** - NVMe and SSD-specific scheduler settings

### 📊 Monitoring & Control
- **Web Dashboard** - Real-time system metrics at `http://localhost:9313`
- **Hardware Sensors** - Temperature, fan speed, and power monitoring
- **Performance Profiles** - Switch between power-saving and performance modes

## System Requirements

- **CPU**: x86_64 processor (Intel/AMD)
- **RAM**: 4GB minimum, 8GB+ recommended
- **Storage**: 25GB for base installation
- **GPU**: Optional but recommended (NVIDIA/AMD/Intel)

## Community

- **GitHub**: [github.com/Arakiss/hecate-os](https://github.com/Arakiss/hecate-os)
- **Issues**: [Report bugs or request features](https://github.com/Arakiss/hecate-os/issues)
- **Discussions**: [Join the conversation](https://github.com/Arakiss/hecate-os/discussions)

## Development

HecateOS is built with modern Rust for performance and reliability. Want to contribute?

- [Contributing Guide](docs/CONTRIBUTING.md)
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Building from Source](docs/BUILDING.md)

## License

MIT License - See [LICENSE](LICENSE) for details.

---

**Based on Ubuntu 24.04 LTS** - HecateOS builds upon the solid foundation of Ubuntu, adding performance optimizations and modern tooling while maintaining full compatibility with the Ubuntu ecosystem.

