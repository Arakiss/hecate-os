# HecateOS

A Linux distribution that detects your hardware and applies specific optimizations automatically. Based on Ubuntu 24.04 LTS.

> **Status: Alpha (v0.1.0)** — Framework complete, tested on one machine. No ISO releases yet.

## The Problem

Most Linux distros ship with generic configs. You install Ubuntu, then spend hours tweaking sysctl, GRUB parameters, GPU drivers, and kernel settings. Or you don't, and leave performance on the table.

## What HecateOS Does

On first boot, HecateOS runs `hardware-detector.sh` which:

1. **Detects your hardware** — CPU model/generation, GPU vendor/model/VRAM, RAM amount/speed, storage type
2. **Creates a profile** — Automatically classifies your system based on capabilities
3. **Applies optimizations** — Sets kernel parameters, sysctl values, GPU settings, I/O schedulers specific to YOUR hardware

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Hardware Detect │ ──▶ │ Auto Profile    │ ──▶ │ Apply Tuning    │
│                 │     │                 │     │                 │
│ • CPU gen       │     │ Based on:       │     │ • sysctl.conf   │
│ • GPU tier      │     │ • GPU VRAM      │     │ • GRUB params   │
│ • RAM amount    │     │ • RAM amount    │     │ • GPU settings  │
│ • Storage type  │     │ • CPU cores     │     │ • I/O scheduler │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## What Gets Tuned

**CPU** — Intel P-State or AMD P-State governor, C-State limits, turbo boost settings

**Memory** — Swappiness (10 for high RAM, 60 for low), dirty ratios, ZRAM compression ratio, transparent hugepages

**GPU (NVIDIA)** — Driver version by generation (570 for RTX 40, 535 for RTX 30, etc.), persistence mode, power limits, compute mode

**Storage** — I/O scheduler (none for NVMe Gen4+, mq-deadline for older), read-ahead values

**Kernel** — `mitigations=off` for ~10% perf gain (configurable), `intel_pstate=active`, IOMMU, PCIe ASPM off

## Tested Hardware

Actually tested on:
- Intel Core i9-13900K
- NVIDIA RTX 4090
- 128GB DDR5-6400
- Samsung 990 PRO NVMe

Should work on (detection logic exists but untested):
- Intel 10th gen+
- AMD Ryzen (Zen 2+)
- NVIDIA GTX 10 series+
- AMD GPUs (basic support)
- 8GB-512GB RAM
- Any NVMe/SATA SSD/HDD

## Project Structure

```
hecate-os/
├── bin/                        # CLI tools (hecate-*)
│   ├── hecate                  # Main dispatcher
│   ├── hecate-info             # System info and status
│   ├── hecate-update           # Update system + migrations
│   ├── hecate-optimize         # Re-apply optimizations
│   ├── hecate-driver           # GPU driver management
│   └── hecate-migrate          # Run pending migrations
├── scripts/
│   ├── hardware-detector.sh    # Detects and profiles hardware
│   ├── apply-optimizations.sh  # Applies profile-specific tuning
│   ├── hecate-driver-installer.sh  # GPU driver selection
│   └── hecate-benchmark.sh     # Performance testing
├── migrations/                 # Timestamped migration scripts
├── config/
│   ├── package-lists/          # Packages to install
│   ├── includes.chroot/        # System configs (sysctl, GRUB, docker)
│   └── hooks/                  # Build-time scripts
├── Dockerfile.build            # Docker build environment
└── build.sh                    # Main build script
```

## CLI Commands

After installation, HecateOS provides the `hecate` command:

```bash
hecate info          # Show system info and applied optimizations
hecate update        # Update system packages and run migrations
hecate optimize      # Re-detect hardware and apply optimizations
hecate driver        # Manage GPU drivers (status/install/remove)
hecate migrate       # Run pending migrations
hecate benchmark     # Run performance benchmark
```

## Building

### Option 1: Docker (Recommended)

```bash
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os

# Build the Docker image and ISO
docker compose run --rm build

# Or manually:
docker build -f Dockerfile.build -t hecate-builder .
docker run --rm --privileged -v $(pwd):/build hecate-builder
```

### Option 2: Native Build

Requires Ubuntu 24.04:

```bash
# Install dependencies
sudo apt install live-build debootstrap squashfs-tools xorriso isolinux

# Clone and build
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os
sudo ./build.sh build
```

ISO output: `iso/hecate-os-0.1.0-amd64-YYYYMMDD.iso`

## Performance Claims

These are estimates based on the optimizations applied. No benchmarks yet.

| Change | Expected Gain | Why |
|--------|---------------|-----|
| `mitigations=off` | 5-15% | Removes Spectre/Meltdown overhead |
| Performance governor | 3-8% | No frequency scaling latency |
| ZRAM vs disk swap | 10-20% | Compression faster than disk I/O |
| Tuned sysctl | 2-5% | Better memory/network/scheduler settings |

Real benchmarks will come after community testing on varied hardware.

## Security Trade-offs

HecateOS prioritizes performance over security hardening:

- Spectre/Meltdown mitigations disabled by default
- SSH enabled by default
- Firewall installed but not enabled

See [SECURITY.md](SECURITY.md) for details and how to re-enable protections.

## Contributing

Need testers with:
- AMD Ryzen CPUs (Zen 2, 3, 4)
- AMD GPUs (RX 6000/7000)
- Laptops (battery/thermal management)
- Lower-end hardware (8GB RAM, older GPUs)

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## Why "HecateOS"?

Named after my cat, who sits at the crossroads between my keyboard and monitor. The Greek goddess Hecate ruled crossroads and magic. This distro lives at the crossroads between Windows dual-boot and Linux, between generic configs and hardware-specific tuning.

## License

MIT. Based on Ubuntu 24.04 LTS by Canonical.

---

**Links:** [Roadmap](docs/ROADMAP.md) · [Security](SECURITY.md) · [Issues](https://github.com/Arakiss/hecate-os/issues)
