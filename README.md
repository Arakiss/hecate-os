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

- **Automatic hardware profiling** - Detects your CPU, GPU, RAM, and storage on first boot
- **Performance optimization** - Applies hardware-specific kernel parameters, drivers, and system settings
- **Real-time monitoring** - WebSocket-based system monitoring with web dashboard
- **Modern tooling** - Rust-based system components for performance-critical operations
- **Package management** - Custom package manager with dependency resolution

## Documentation

- [Hardware Compatibility](docs/HARDWARE.md)
- [Building from Source](docs/BUILDING.md)
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Security Considerations](SECURITY.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [Roadmap](docs/ROADMAP.md)

## License

MIT License - See [LICENSE](LICENSE) file for details.

Based on Ubuntu 24.04 LTS by Canonical Ltd.

## Links

- [Issues](https://github.com/Arakiss/hecate-os/issues)
- [Discussions](https://github.com/Arakiss/hecate-os/discussions)