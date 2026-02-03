# HecateOS

> ⚠️ **ALPHA SOFTWARE** - This project is under active development and not ready for production use. Expect breaking changes.

Performance-optimized Linux distribution with automatic hardware detection and tuning. Based on Ubuntu 24.04 LTS.

## Quick Start

### Building the ISO

```bash
# Using Docker (recommended)
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os
docker compose run --rm build

# ISO will be in: iso/hecate-os-*.iso
```

### Installation

1. Write ISO to USB drive
2. Boot from USB
3. System automatically detects hardware and applies optimizations
4. Reboot to enjoy tuned system

## Features

### ✅ Production Ready
- **Automatic hardware profiling** - Detects your CPU, GPU, RAM, and storage on first boot
- **Performance optimization** - Applies hardware-specific kernel parameters, drivers, and system settings
- **Advanced GPU management** - Multi-vendor support (NVIDIA/AMD), VRAM monitoring, load balancing
- **ML workload optimization** - Auto-tunes PyTorch/TensorFlow, batch sizes, and distributed training
- **Real-time monitoring** - WebSocket-based dashboard with system metrics and thermal monitoring
- **Comprehensive benchmarking** - CPU, GPU, memory, disk, network, and AI/ML performance testing
- **Modern tooling** - Rust-based system components with < 50MB RAM footprint

### 🚧 In Development
- **Package management** - Native package manager with parallel downloads (60% complete)
- **Intelligent updates** - Live kernel patching and driver hot-swapping (planned)

## Rust Components Status

| Component | Status | Description |
|-----------|--------|-------------|
| `hecate-daemon` | ✅ Production | System optimization daemon with hardware detection |
| `hecate-gpu` | ✅ Production | Advanced GPU management with 26+ tests |
| `hecate-ml` | ✅ Production | ML workload optimizer for PyTorch/TensorFlow |
| `hecate-monitor` | ✅ Production | Real-time performance dashboard (port 9313) |
| `hecate-bench` | ✅ Production | Comprehensive benchmarking suite |
| `hecate-core` | ✅ Production | Core hardware detection library |
| `hecate-pkg` | ⚠️ 60% Complete | Package manager (needs database implementation) |
| `hecate-update` | ❌ Planned | Intelligent update system |

## Documentation

- [Hardware Compatibility](docs/HARDWARE.md)
- [Building from Source](docs/BUILDING.md)
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Development Tools](docs/DEVELOPMENT-TOOLS.md)
- [Security Considerations](SECURITY.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [Roadmap](docs/ROADMAP.md) - **Updated February 2025**

## License

MIT License - See [LICENSE](LICENSE) file for details.

Based on Ubuntu 24.04 LTS by Canonical Ltd.

## Links

- [Issues](https://github.com/Arakiss/hecate-os/issues)
- [Discussions](https://github.com/Arakiss/hecate-os/discussions)