# HecateOS Editions

HecateOS comes in multiple editions optimized for different use cases and hardware configurations.

## 🚀 Available Editions

### 1. **HecateOS Ultimate** (Flagship Edition)
**Target:** High-end AI/ML workstations with top-tier hardware
- **Minimum RAM:** 64GB (optimized for 128GB+)
- **GPU:** NVIDIA RTX 4080/4090 or better
- **Storage:** NVMe Gen4/Gen5
- **Includes:** Full CUDA stack, AI/ML frameworks, all optimizations
- **ISO Size:** ~6GB
- **Use Cases:** AI research, deep learning, 3D rendering, scientific computing

### 2. **HecateOS Workstation**
**Target:** Professional workstations and power users
- **Minimum RAM:** 32GB
- **GPU:** NVIDIA RTX 3060 or better
- **Storage:** Any NVMe SSD
- **Includes:** NVIDIA drivers, Docker, development tools
- **ISO Size:** ~4GB
- **Use Cases:** Software development, content creation, CAD/CAM

### 3. **HecateOS Gaming**
**Target:** Gaming rigs and streaming setups
- **Minimum RAM:** 16GB
- **GPU:** NVIDIA GTX 1660 or better
- **Storage:** Any SSD
- **Includes:** Gaming optimizations, low-latency kernel, Steam/Lutris ready
- **ISO Size:** ~3.5GB
- **Use Cases:** Gaming, streaming, content creation

### 4. **HecateOS Developer**
**Target:** Development machines
- **Minimum RAM:** 16GB
- **GPU:** Optional (integrated graphics OK)
- **Storage:** 256GB+ SSD
- **Includes:** All programming languages, containers, databases
- **ISO Size:** ~3GB
- **Use Cases:** Web development, backend services, DevOps

### 5. **HecateOS Lite**
**Target:** Standard computers and older hardware
- **Minimum RAM:** 8GB
- **GPU:** Any (integrated OK)
- **Storage:** 128GB+
- **Includes:** Core system, basic tools, no NVIDIA drivers
- **ISO Size:** ~2GB
- **Use Cases:** General computing, office work, web browsing

### 6. **HecateOS Server**
**Target:** Headless servers and compute nodes
- **Minimum RAM:** 8GB
- **GPU:** Optional
- **Storage:** Any
- **Includes:** Server stack, no GUI, container runtime
- **ISO Size:** ~1.5GB
- **Use Cases:** Web servers, databases, microservices, homelab

## 📊 Edition Comparison Matrix

| Feature | Ultimate | Workstation | Gaming | Developer | Lite | Server |
|---------|----------|-------------|--------|-----------|------|--------|
| NVIDIA Drivers | ✅ Latest | ✅ Stable | ✅ Gaming | Optional | ❌ | Optional |
| CUDA/cuDNN | ✅ Full | ✅ Basic | ❌ | Optional | ❌ | Optional |
| AI/ML Stack | ✅ Complete | ✅ Basic | ❌ | ✅ Basic | ❌ | ❌ |
| Development Tools | ✅ All | ✅ Most | ✅ Basic | ✅ All | ✅ Basic | ✅ CLI |
| Gaming Tools | ✅ | ✅ | ✅ Optimized | ❌ | ❌ | ❌ |
| Desktop Environment | Optional | ✅ | ✅ | ✅ | ✅ | ❌ |
| Container Runtime | ✅ GPU | ✅ GPU | ✅ | ✅ | ✅ | ✅ |
| Databases | ✅ All | ✅ Most | ❌ | ✅ All | ❌ | ✅ |
| Performance Tuning | Extreme | High | Gaming | Balanced | Standard | Server |
| Kernel | RT/Low-latency | Low-latency | Gaming | Generic | Generic | Server |

## 🔧 Auto-Detection

During installation, HecateOS automatically:
1. **Detects your hardware** (CPU, GPU, RAM, Storage)
2. **Recommends the best edition** for your system
3. **Allows manual override** if desired
4. **Applies optimal settings** based on detected hardware

## 💿 Download the Right Edition

The installer will automatically detect and recommend the best edition for your hardware. However, you can manually select any edition during installation.

### Quick Selection Guide:
- **Have RTX 4090 + 128GB RAM?** → Ultimate
- **Professional workstation?** → Workstation
- **Gaming PC?** → Gaming
- **Coding machine?** → Developer
- **Basic computer?** → Lite
- **Headless server?** → Server

## 🔄 Switching Editions

You can switch between editions after installation using:
```bash
sudo hecate-edition switch <edition-name>
```

This will:
- Install/remove packages as needed
- Adjust system optimizations
- Update kernel if required
- Preserve your data and settings