# Building HecateOS

## Prerequisites

- Ubuntu 24.04 LTS (for native build) or Docker
- At least 20GB free disk space
- 8GB+ RAM recommended

## Building the ISO

### Option 1: Docker Build (Recommended)

```bash
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os

# Build using Docker Compose
docker compose run --rm build

# Or manually with Docker
docker build -f Dockerfile.build -t hecate-builder .
docker run --rm --privileged -v $(pwd):/build hecate-builder
```

The ISO will be created in: `iso/hecate-os-0.2.0-amd64-YYYYMMDD.iso`

### Option 2: Native Build

Requires Ubuntu 24.04:

```bash
# Install dependencies
sudo apt update
sudo apt install -y \
    live-build \
    debootstrap \
    squashfs-tools \
    xorriso \
    isolinux \
    syslinux-utils

# Clone and build
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os
sudo ./build.sh build
```

## Building Rust Components

The Rust components can be built separately:

```bash
cd rust

# Build all components
for dir in hecate-*; do
    cd $dir
    cargo build --release
    cd ..
done

# Or build specific component
cd hecate-monitor
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test --all

# Run benchmarks
cargo bench --all

# Run with coverage
cargo tarpaulin --all
```

## Building the Web Dashboard

```bash
cd hecate-dashboard

# Install dependencies (using Bun)
bun install

# Development server
bun run dev

# Production build
bun run build
```

## CI/CD Pipeline

The project includes GitHub Actions workflows for:

- Rust component builds and tests
- Multi-architecture support (x86_64, aarch64)
- Security audits
- Code quality checks

See `.github/workflows/` for pipeline definitions.

## Release Process

1. Update version in `CHANGELOG.md`
2. Create git tag: `git tag -a v0.2.0 -m "Release v0.2.0"`
3. Push tag: `git push origin v0.2.0`
4. GitHub Actions will build and create release artifacts

## Troubleshooting

### Build Fails with 404 Errors
Ubuntu mirrors might be out of sync. Wait a few hours and retry.

### Docker Build Permission Denied
Ensure Docker daemon is running and you have proper permissions:
```bash
sudo usermod -aG docker $USER
newgrp docker
```

### Out of Disk Space
The build process requires significant space:
- Docker images: ~5GB
- Build cache: ~10GB
- Final ISO: ~3GB

Clean up with:
```bash
docker system prune -a
sudo lb clean --purge
```