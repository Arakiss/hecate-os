# HecateOS - IMPORTANT PROJECT CONTEXT

## ⚠️ CRITICAL: What HecateOS Actually Is

**HecateOS is a FULL LINUX DISTRIBUTION based on Ubuntu LTS**

This is NOT:
- ❌ A collection of standalone tools
- ❌ A set of binaries that run on any system
- ❌ An application suite
- ❌ Something that works without Ubuntu

This IS:
- ✅ A complete Linux operating system distribution
- ✅ Based on Ubuntu 24.04 LTS (or 22.04 LTS)
- ✅ A modified Ubuntu with performance optimizations
- ✅ A bootable operating system with kernel, drivers, etc.
- ✅ Requires Ubuntu ISO as base to build

## Core Architecture

HecateOS = Ubuntu LTS Base + HecateOS Optimizations + Custom Tools

The distribution includes:
1. **Ubuntu LTS Foundation**: Full Ubuntu kernel, systemd, drivers, libraries
2. **HecateOS Daemon (hecated)**: System-level performance optimizer
3. **HecateOS Tools**: Suite of optimization and monitoring tools
4. **Custom Configurations**: Optimized system settings and kernel parameters

## Build Requirements (MANDATORY)

To build HecateOS ISO you MUST have:
1. **Ubuntu ISO** (24.04 or 22.04 LTS) - The base operating system
2. **7z** (p7zip-full) - To extract and modify Ubuntu ISO
3. **Rust toolchain** - To build HecateOS components

```bash
# REQUIRED: Install 7z
sudo apt-get install p7zip-full

# Build HecateOS distribution
hecate-dev iso create --download 24.04
```

## What Gets Built

When you create a HecateOS ISO, you get:
- Full Ubuntu Linux operating system
- Linux kernel with optimizations
- All Ubuntu packages and dependencies
- HecateOS daemon integrated with systemd
- HecateOS tools pre-installed
- Custom GRUB bootloader configuration
- Installer that sets up the complete OS

## Development Workflow

1. **Development**: Work on HecateOS tools in Rust
2. **Testing**: Test tools on your current Ubuntu system
3. **Integration**: Build ISO that integrates tools into Ubuntu
4. **Distribution**: ISO creates a full bootable Linux OS

## Key Points for AI/Development

- NEVER suggest creating "standalone" ISOs without Ubuntu
- ALWAYS require Ubuntu ISO as base
- The project CANNOT function without Ubuntu
- 7z is MANDATORY for ISO creation (not optional)
- HecateOS tools alone are useless without the OS
- This is OS development, not application development

## ISO Creation Process

1. Download Ubuntu 24.04/22.04 LTS ISO
2. Extract ISO contents (requires 7z)
3. Inject HecateOS components into Ubuntu filesystem
4. Configure systemd services for hecated
5. Modify GRUB for optimized boot
6. Add HecateOS configuration files
7. Repack as bootable HecateOS ISO

## Target Users

HecateOS is for users who want:
- A performance-optimized Linux distribution
- Ubuntu compatibility with better performance
- Gaming/workstation optimizations out of the box
- Not just tools, but a complete operating system

## Remember

**HecateOS without Ubuntu is like a car engine without a car - completely useless.**

The entire point is to create a better Ubuntu-based distribution, not a collection of tools.