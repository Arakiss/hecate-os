#!/bin/bash
#
# HecateOS Hardware Detection System
# Automatically detects and optimizes for various hardware configurations
#

set -e

# Output file for hardware profile
PROFILE_FILE="/etc/hecate/hardware-profile.json"
OPTIMIZATION_FILE="/etc/hecate/optimizations.conf"

# Create directory if doesn't exist
mkdir -p /etc/hecate

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

echo -e "${CYAN}HecateOS Hardware Detection System${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Function to detect CPU
detect_cpu() {
    echo -e "\n${YELLOW}Detecting CPU...${RESET}"
    
    CPU_MODEL=$(lscpu | grep "Model name" | cut -d: -f2 | xargs)
    CPU_VENDOR=$(lscpu | grep "Vendor ID" | cut -d: -f2 | xargs)
    CPU_CORES=$(nproc)
    CPU_THREADS=$(lscpu | grep "^CPU(s):" | cut -d: -f2 | xargs)
    CPU_ARCH=$(uname -m)
    
    # Detect CPU generation and type
    CPU_GENERATION="unknown"
    CPU_TYPE="standard"
    P_CORES=""
    E_CORES=""
    
    if [[ "$CPU_VENDOR" == "GenuineIntel" ]]; then
        if [[ "$CPU_MODEL" =~ "13th Gen" ]] || [[ "$CPU_MODEL" =~ "i9-13" ]] || [[ "$CPU_MODEL" =~ "i7-13" ]]; then
            CPU_GENERATION="intel-13th"
            CPU_TYPE="hybrid"
            # Detect P-cores and E-cores for 13th gen
            if [[ "$CPU_MODEL" =~ "i9-13900" ]]; then
                P_CORES="0-15"  # 8 P-cores with HT
                E_CORES="16-31"  # 16 E-cores
            elif [[ "$CPU_MODEL" =~ "i7-13700" ]]; then
                P_CORES="0-15"  # 8 P-cores with HT
                E_CORES="16-23"  # 8 E-cores
            fi
        elif [[ "$CPU_MODEL" =~ "12th Gen" ]] || [[ "$CPU_MODEL" =~ "i9-12" ]] || [[ "$CPU_MODEL" =~ "i7-12" ]]; then
            CPU_GENERATION="intel-12th"
            CPU_TYPE="hybrid"
            if [[ "$CPU_MODEL" =~ "i9-12900" ]]; then
                P_CORES="0-15"
                E_CORES="16-23"
            fi
        elif [[ "$CPU_MODEL" =~ "11th Gen" ]] || [[ "$CPU_MODEL" =~ "i9-11" ]]; then
            CPU_GENERATION="intel-11th"
        elif [[ "$CPU_MODEL" =~ "10th Gen" ]] || [[ "$CPU_MODEL" =~ "i9-10" ]]; then
            CPU_GENERATION="intel-10th"
        fi
    elif [[ "$CPU_VENDOR" == "AuthenticAMD" ]]; then
        if [[ "$CPU_MODEL" =~ "Ryzen 9 7" ]]; then
            CPU_GENERATION="amd-zen4"
        elif [[ "$CPU_MODEL" =~ "Ryzen 9 5" ]]; then
            CPU_GENERATION="amd-zen3"
        elif [[ "$CPU_MODEL" =~ "Ryzen 9 3" ]]; then
            CPU_GENERATION="amd-zen2"
        fi
    fi
    
    echo -e "${GREEN}✓${RESET} CPU: $CPU_MODEL"
    echo -e "${GREEN}✓${RESET} Architecture: $CPU_ARCH"
    echo -e "${GREEN}✓${RESET} Cores/Threads: $CPU_CORES/$CPU_THREADS"
    echo -e "${GREEN}✓${RESET} Generation: $CPU_GENERATION"
    
    # Export to JSON
    cat >> "$PROFILE_FILE" << EOF
{
  "cpu": {
    "model": "$CPU_MODEL",
    "vendor": "$CPU_VENDOR",
    "cores": $CPU_CORES,
    "threads": $CPU_THREADS,
    "architecture": "$CPU_ARCH",
    "generation": "$CPU_GENERATION",
    "type": "$CPU_TYPE",
    "p_cores": "$P_CORES",
    "e_cores": "$E_CORES"
  },
EOF
}

# Function to detect memory
detect_memory() {
    echo -e "\n${YELLOW}Detecting Memory...${RESET}"
    
    TOTAL_MEM=$(free -b | grep "Mem:" | awk '{print $2}')
    TOTAL_MEM_GB=$((TOTAL_MEM / 1073741824))
    
    # Detect memory speed if available
    MEM_SPEED="unknown"
    if command -v dmidecode &> /dev/null; then
        MEM_SPEED=$(dmidecode -t memory | grep "Speed:" | grep "MHz" | head -1 | awk '{print $2}')
        MEM_TYPE=$(dmidecode -t memory | grep "Type:" | grep -v "Unknown" | head -1 | awk '{print $2}')
    fi
    
    echo -e "${GREEN}✓${RESET} Total RAM: ${TOTAL_MEM_GB}GB"
    echo -e "${GREEN}✓${RESET} Memory Type: $MEM_TYPE @ ${MEM_SPEED}MHz"
    
    # Determine memory tier
    MEM_TIER="standard"
    if [ $TOTAL_MEM_GB -ge 128 ]; then
        MEM_TIER="extreme"
    elif [ $TOTAL_MEM_GB -ge 64 ]; then
        MEM_TIER="high"
    elif [ $TOTAL_MEM_GB -ge 32 ]; then
        MEM_TIER="medium"
    fi
    
    cat >> "$PROFILE_FILE" << EOF
  "memory": {
    "total_gb": $TOTAL_MEM_GB,
    "total_bytes": $TOTAL_MEM,
    "type": "$MEM_TYPE",
    "speed": "$MEM_SPEED",
    "tier": "$MEM_TIER"
  },
EOF
}

# Function to detect GPU
detect_gpu() {
    echo -e "\n${YELLOW}Detecting GPU...${RESET}"
    
    GPU_VENDOR="none"
    GPU_MODEL="none"
    GPU_VRAM="0"
    GPU_DRIVER="none"
    GPU_TIER="none"
    
    # Check for NVIDIA GPU
    if lspci | grep -i nvidia > /dev/null; then
        GPU_VENDOR="nvidia"
        
        if command -v nvidia-smi &> /dev/null; then
            GPU_MODEL=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
            GPU_VRAM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)
            GPU_DRIVER=$(nvidia-smi --query-gpu=driver_version --format=csv,noheader | head -1)
            
            # Determine GPU tier
            if [[ "$GPU_MODEL" =~ "RTX 4090" ]] || [[ "$GPU_MODEL" =~ "RTX 4080" ]]; then
                GPU_TIER="flagship"
            elif [[ "$GPU_MODEL" =~ "RTX 40" ]]; then
                GPU_TIER="high-end"
            elif [[ "$GPU_MODEL" =~ "RTX 30" ]]; then
                GPU_TIER="previous-gen-high"
            elif [[ "$GPU_MODEL" =~ "RTX 20" ]]; then
                GPU_TIER="older-gen"
            elif [[ "$GPU_MODEL" =~ "GTX" ]]; then
                GPU_TIER="legacy"
            else
                GPU_TIER="professional"
            fi
        else
            GPU_MODEL=$(lspci | grep -i nvidia | head -1)
        fi
        
        echo -e "${GREEN}✓${RESET} GPU: $GPU_MODEL"
        echo -e "${GREEN}✓${RESET} VRAM: ${GPU_VRAM}MB"
        echo -e "${GREEN}✓${RESET} Driver: $GPU_DRIVER"
        
    # Check for AMD GPU
    elif lspci | grep -i "amd.*vga\|amd.*display" > /dev/null; then
        GPU_VENDOR="amd"
        GPU_MODEL=$(lspci | grep -i "amd.*vga\|amd.*display" | cut -d: -f3)
        echo -e "${GREEN}✓${RESET} GPU: AMD $GPU_MODEL"
        GPU_TIER="amd-gpu"
        
    # Check for Intel GPU
    elif lspci | grep -i "intel.*graphics" > /dev/null; then
        GPU_VENDOR="intel"
        GPU_MODEL=$(lspci | grep -i "intel.*graphics" | cut -d: -f3)
        echo -e "${GREEN}✓${RESET} GPU: Intel $GPU_MODEL"
        GPU_TIER="integrated"
    else
        echo -e "${YELLOW}⚠${RESET} No dedicated GPU detected"
    fi
    
    cat >> "$PROFILE_FILE" << EOF
  "gpu": {
    "vendor": "$GPU_VENDOR",
    "model": "$GPU_MODEL",
    "vram_mb": $GPU_VRAM,
    "driver": "$GPU_DRIVER",
    "tier": "$GPU_TIER"
  },
EOF
}

# Function to detect storage
detect_storage() {
    echo -e "\n${YELLOW}Detecting Storage...${RESET}"
    
    NVME_COUNT=$(ls /dev/nvme* 2>/dev/null | grep -c "nvme[0-9]n1$" || echo 0)
    SSD_COUNT=$(lsblk -d -o NAME,ROTA | grep " 0$" | grep -c "^sd" || echo 0)
    HDD_COUNT=$(lsblk -d -o NAME,ROTA | grep " 1$" | grep -c "^sd" || echo 0)
    
    echo -e "${GREEN}✓${RESET} NVMe drives: $NVME_COUNT"
    echo -e "${GREEN}✓${RESET} SATA SSDs: $SSD_COUNT"
    echo -e "${GREEN}✓${RESET} HDDs: $HDD_COUNT"
    
    # Detect NVMe generation
    NVME_GEN="unknown"
    if [ $NVME_COUNT -gt 0 ]; then
        for nvme in /dev/nvme*n1; do
            if [ -e "$nvme" ]; then
                LINK_SPEED=$(nvme id-ctrl "$nvme" 2>/dev/null | grep "pci_link_speed" || echo "")
                if [[ "$LINK_SPEED" =~ "32.0 GT/s" ]]; then
                    NVME_GEN="gen5"
                    echo -e "${GREEN}✓${RESET} NVMe Generation: PCIe 5.0 detected!"
                elif [[ "$LINK_SPEED" =~ "16.0 GT/s" ]]; then
                    NVME_GEN="gen4"
                    echo -e "${GREEN}✓${RESET} NVMe Generation: PCIe 4.0"
                else
                    NVME_GEN="gen3"
                fi
                break
            fi
        done
    fi
    
    cat >> "$PROFILE_FILE" << EOF
  "storage": {
    "nvme_count": $NVME_COUNT,
    "ssd_count": $SSD_COUNT,
    "hdd_count": $HDD_COUNT,
    "nvme_generation": "$NVME_GEN"
  },
EOF
}

# Function to detect network
detect_network() {
    echo -e "\n${YELLOW}Detecting Network...${RESET}"
    
    ETH_COUNT=$(ip link show | grep -c "^[0-9]*: en" || echo 0)
    WIFI_COUNT=$(ip link show | grep -c "^[0-9]*: wl" || echo 0)
    
    # Check for high-speed ethernet
    ETH_SPEED="1Gbps"
    if ethtool 2>/dev/null | grep "10000baseT" > /dev/null; then
        ETH_SPEED="10Gbps"
    elif ethtool 2>/dev/null | grep "2500baseT" > /dev/null; then
        ETH_SPEED="2.5Gbps"
    fi
    
    echo -e "${GREEN}✓${RESET} Ethernet interfaces: $ETH_COUNT ($ETH_SPEED capable)"
    echo -e "${GREEN}✓${RESET} WiFi interfaces: $WIFI_COUNT"
    
    cat >> "$PROFILE_FILE" << EOF
  "network": {
    "ethernet_count": $ETH_COUNT,
    "wifi_count": $WIFI_COUNT,
    "ethernet_speed": "$ETH_SPEED"
  },
EOF
}

# Function to determine system profile
determine_profile() {
    echo -e "\n${YELLOW}Determining System Profile...${RESET}"
    
    # Read the profile data
    source <(jq -r '.cpu | to_entries[] | "CPU_\(.key | ascii_upcase)=\(.value)"' "$PROFILE_FILE" 2>/dev/null || true)
    source <(jq -r '.memory | to_entries[] | "MEM_\(.key | ascii_upcase)=\(.value)"' "$PROFILE_FILE" 2>/dev/null || true)
    source <(jq -r '.gpu | to_entries[] | "GPU_\(.key | ascii_upcase)=\(.value)"' "$PROFILE_FILE" 2>/dev/null || true)
    
    SYSTEM_PROFILE="standard"
    SYSTEM_CLASS="workstation"
    
    # Determine system profile based on hardware
    if [[ "$GPU_TIER" == "flagship" ]] && [[ "$MEM_TIER" == "extreme" ]]; then
        SYSTEM_PROFILE="ultimate"
        SYSTEM_CLASS="ai-workstation"
        echo -e "${GREEN}✓${RESET} Profile: ${CYAN}ULTIMATE AI WORKSTATION${RESET}"
    elif [[ "$GPU_TIER" == "flagship" ]] || [[ "$GPU_TIER" == "high-end" ]]; then
        SYSTEM_PROFILE="gaming"
        SYSTEM_CLASS="gaming-workstation"
        echo -e "${GREEN}✓${RESET} Profile: ${CYAN}GAMING/CREATIVE WORKSTATION${RESET}"
    elif [[ "$GPU_VENDOR" == "nvidia" ]] && [[ "$MEM_TOTAL_GB" -ge 32 ]]; then
        SYSTEM_PROFILE="developer"
        SYSTEM_CLASS="dev-workstation"
        echo -e "${GREEN}✓${RESET} Profile: ${CYAN}DEVELOPER WORKSTATION${RESET}"
    elif [[ "$GPU_VENDOR" == "none" ]] || [[ "$GPU_TIER" == "integrated" ]]; then
        if [[ "$CPU_THREADS" -ge 16 ]]; then
            SYSTEM_PROFILE="server"
            SYSTEM_CLASS="compute-server"
            echo -e "${GREEN}✓${RESET} Profile: ${CYAN}COMPUTE SERVER${RESET}"
        else
            SYSTEM_PROFILE="minimal"
            SYSTEM_CLASS="basic-system"
            echo -e "${GREEN}✓${RESET} Profile: ${CYAN}BASIC SYSTEM${RESET}"
        fi
    fi
    
    # Add profile to JSON
    cat >> "$PROFILE_FILE" << EOF
  "system": {
    "profile": "$SYSTEM_PROFILE",
    "class": "$SYSTEM_CLASS",
    "hostname": "$(hostname)",
    "kernel": "$(uname -r)",
    "distro": "HecateOS",
    "version": "24.04"
  }
}
EOF
    
    echo -e "\n${CYAN}Hardware profile saved to: $PROFILE_FILE${RESET}"
}

# Function to generate optimizations
generate_optimizations() {
    echo -e "\n${YELLOW}Generating Optimizations...${RESET}"
    
    # Read profile
    PROFILE=$(cat "$PROFILE_FILE")
    SYSTEM_PROFILE=$(echo "$PROFILE" | jq -r '.system.profile')
    CPU_TYPE=$(echo "$PROFILE" | jq -r '.cpu.type')
    CPU_THREADS=$(echo "$PROFILE" | jq -r '.cpu.threads')
    MEM_GB=$(echo "$PROFILE" | jq -r '.memory.total_gb')
    GPU_VENDOR=$(echo "$PROFILE" | jq -r '.gpu.vendor')
    GPU_TIER=$(echo "$PROFILE" | jq -r '.gpu.tier')
    NVME_GEN=$(echo "$PROFILE" | jq -r '.storage.nvme_generation')
    
    # Start optimization file
    cat > "$OPTIMIZATION_FILE" << EOF
# HecateOS Automatic Optimizations
# Generated: $(date)
# System Profile: $SYSTEM_PROFILE

EOF
    
    # CPU Optimizations
    echo "# CPU Optimizations" >> "$OPTIMIZATION_FILE"
    if [[ "$CPU_TYPE" == "hybrid" ]]; then
        echo "CPU_SCHEDULER=schedutil" >> "$OPTIMIZATION_FILE"
        echo "INTEL_PSTATE=active" >> "$OPTIMIZATION_FILE"
        echo "CPU_BOOST=1" >> "$OPTIMIZATION_FILE"
    else
        echo "CPU_SCHEDULER=performance" >> "$OPTIMIZATION_FILE"
        echo "CPU_BOOST=1" >> "$OPTIMIZATION_FILE"
    fi
    
    # Memory Optimizations
    echo -e "\n# Memory Optimizations" >> "$OPTIMIZATION_FILE"
    if [ $MEM_GB -ge 128 ]; then
        echo "VM_SWAPPINESS=5" >> "$OPTIMIZATION_FILE"
        echo "VM_DIRTY_RATIO=60" >> "$OPTIMIZATION_FILE"
        echo "ZRAM_SIZE=25" >> "$OPTIMIZATION_FILE"
    elif [ $MEM_GB -ge 64 ]; then
        echo "VM_SWAPPINESS=10" >> "$OPTIMIZATION_FILE"
        echo "VM_DIRTY_RATIO=40" >> "$OPTIMIZATION_FILE"
        echo "ZRAM_SIZE=25" >> "$OPTIMIZATION_FILE"
    elif [ $MEM_GB -ge 32 ]; then
        echo "VM_SWAPPINESS=20" >> "$OPTIMIZATION_FILE"
        echo "VM_DIRTY_RATIO=30" >> "$OPTIMIZATION_FILE"
        echo "ZRAM_SIZE=50" >> "$OPTIMIZATION_FILE"
    else
        echo "VM_SWAPPINESS=60" >> "$OPTIMIZATION_FILE"
        echo "VM_DIRTY_RATIO=20" >> "$OPTIMIZATION_FILE"
        echo "ZRAM_SIZE=75" >> "$OPTIMIZATION_FILE"
    fi
    
    # GPU Optimizations
    echo -e "\n# GPU Optimizations" >> "$OPTIMIZATION_FILE"
    if [[ "$GPU_VENDOR" == "nvidia" ]]; then
        echo "NVIDIA_PERSISTENCE=1" >> "$OPTIMIZATION_FILE"
        if [[ "$GPU_TIER" == "flagship" ]]; then
            echo "NVIDIA_POWER_LIMIT=450" >> "$OPTIMIZATION_FILE"
            echo "NVIDIA_CLOCK_OFFSET=200" >> "$OPTIMIZATION_FILE"
        elif [[ "$GPU_TIER" == "high-end" ]]; then
            echo "NVIDIA_POWER_LIMIT=350" >> "$OPTIMIZATION_FILE"
            echo "NVIDIA_CLOCK_OFFSET=100" >> "$OPTIMIZATION_FILE"
        else
            echo "NVIDIA_POWER_LIMIT=default" >> "$OPTIMIZATION_FILE"
            echo "NVIDIA_CLOCK_OFFSET=0" >> "$OPTIMIZATION_FILE"
        fi
    fi
    
    # Storage Optimizations
    echo -e "\n# Storage Optimizations" >> "$OPTIMIZATION_FILE"
    if [[ "$NVME_GEN" == "gen5" ]]; then
        echo "IO_SCHEDULER=none" >> "$OPTIMIZATION_FILE"
        echo "READAHEAD_KB=256" >> "$OPTIMIZATION_FILE"
    elif [[ "$NVME_GEN" == "gen4" ]]; then
        echo "IO_SCHEDULER=none" >> "$OPTIMIZATION_FILE"
        echo "READAHEAD_KB=128" >> "$OPTIMIZATION_FILE"
    else
        echo "IO_SCHEDULER=mq-deadline" >> "$OPTIMIZATION_FILE"
        echo "READAHEAD_KB=128" >> "$OPTIMIZATION_FILE"
    fi
    
    # Profile-specific optimizations
    echo -e "\n# Profile-Specific Settings" >> "$OPTIMIZATION_FILE"
    case "$SYSTEM_PROFILE" in
        ultimate)
            echo "TUNED_PROFILE=latency-performance" >> "$OPTIMIZATION_FILE"
            echo "KERNEL_PREEMPT=voluntary" >> "$OPTIMIZATION_FILE"
            echo "TRANSPARENT_HUGEPAGE=always" >> "$OPTIMIZATION_FILE"
            ;;
        gaming)
            echo "TUNED_PROFILE=latency-performance" >> "$OPTIMIZATION_FILE"
            echo "KERNEL_PREEMPT=full" >> "$OPTIMIZATION_FILE"
            echo "TRANSPARENT_HUGEPAGE=madvise" >> "$OPTIMIZATION_FILE"
            ;;
        developer)
            echo "TUNED_PROFILE=throughput-performance" >> "$OPTIMIZATION_FILE"
            echo "KERNEL_PREEMPT=voluntary" >> "$OPTIMIZATION_FILE"
            echo "TRANSPARENT_HUGEPAGE=madvise" >> "$OPTIMIZATION_FILE"
            ;;
        server)
            echo "TUNED_PROFILE=throughput-performance" >> "$OPTIMIZATION_FILE"
            echo "KERNEL_PREEMPT=none" >> "$OPTIMIZATION_FILE"
            echo "TRANSPARENT_HUGEPAGE=never" >> "$OPTIMIZATION_FILE"
            ;;
        *)
            echo "TUNED_PROFILE=balanced" >> "$OPTIMIZATION_FILE"
            echo "KERNEL_PREEMPT=voluntary" >> "$OPTIMIZATION_FILE"
            echo "TRANSPARENT_HUGEPAGE=madvise" >> "$OPTIMIZATION_FILE"
            ;;
    esac
    
    echo -e "${GREEN}✓${RESET} Optimizations saved to: $OPTIMIZATION_FILE"
}

# Main execution
main() {
    echo "" > "$PROFILE_FILE"  # Clear file
    
    detect_cpu
    detect_memory
    detect_gpu
    detect_storage
    detect_network
    determine_profile
    generate_optimizations
    
    echo -e "\n${GREEN}Hardware detection complete!${RESET}"
    echo -e "${CYAN}Profile: $PROFILE_FILE${RESET}"
    echo -e "${CYAN}Optimizations: $OPTIMIZATION_FILE${RESET}"
}

# Run if not sourced
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi