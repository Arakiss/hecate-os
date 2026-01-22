# Changelog

All notable changes to HecateOS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with live-build configuration
- Hardware detection system (`hardware-detector.sh`)
- Automatic optimization application (`apply-optimizations.sh`)
- NVIDIA driver installer with GPU tier detection
- Benchmark suite for performance testing
- Single ISO with automatic hardware profiling
- Comprehensive package lists for development, AI/ML, and performance tools
- GRUB theme customization
- Docker daemon pre-configuration
- Systemd services for NVIDIA persistence and IRQ affinity

### Changed
- GitHub Actions updated to v4 (cache, upload-artifact, download-artifact)

### Fixed
- CI pipeline now works (deprecated actions v3 → v4)

## [0.1.0] - Unreleased

First alpha release. Hardware detection and optimization framework complete.

### Tested Hardware
- Intel Core i9-13900K
- NVIDIA RTX 4090
- 128GB DDR5-6400

### Known Limitations
- AMD support is theoretical (untested)
- Laptop optimizations not implemented
- Only high-end Intel/NVIDIA hardware tested

[Unreleased]: https://github.com/Arakiss/hecate-os/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Arakiss/hecate-os/releases/tag/v0.1.0
