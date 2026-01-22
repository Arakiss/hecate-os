#!/bin/bash
#
# HecateOS Optimization Application System
# Applies hardware-specific optimizations based on detected profile
#

set -e

OPTIMIZATION_FILE="/etc/hecate/optimizations.conf"
PROFILE_FILE="/etc/hecate/hardware-profile.json"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
RESET='\033[0m'

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}This script must be run as root${RESET}"
    exit 1
fi

echo -e "${CYAN}HecateOS Optimization Application System${RESET}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if optimization file exists
if [ ! -f "$OPTIMIZATION_FILE" ]; then
    echo -e "${YELLOW}No optimization profile found. Running hardware detection...${RESET}"
    /usr/local/bin/hecate-hardware-detect
fi

# Source optimizations
source "$OPTIMIZATION_FILE"

# Read system profile
SYSTEM_PROFILE=$(jq -r '.system.profile' "$PROFILE_FILE" 2>/dev/null || echo "standard")
echo -e "\n${CYAN}Applying optimizations for profile: ${SYSTEM_PROFILE}${RESET}\n"

# Function to apply CPU optimizations
apply_cpu_optimizations() {
    echo -e "${YELLOW}Applying CPU optimizations...${RESET}"
    
    # Set CPU governor
    if [ ! -z "$CPU_SCHEDULER" ]; then
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            echo "$CPU_SCHEDULER" > $cpu 2>/dev/null || true
        done
        echo -e "${GREEN}✓${RESET} CPU governor set to: $CPU_SCHEDULER"
    fi
    
    # Intel P-State
    if [ ! -z "$INTEL_PSTATE" ] && [ -f /sys/devices/system/cpu/intel_pstate/status ]; then
        echo "$INTEL_PSTATE" > /sys/devices/system/cpu/intel_pstate/status
        echo -e "${GREEN}✓${RESET} Intel P-State set to: $INTEL_PSTATE"
    fi
    
    # CPU Boost
    if [ ! -z "$CPU_BOOST" ]; then
        if [ -f /sys/devices/system/cpu/cpufreq/boost ]; then
            echo "$CPU_BOOST" > /sys/devices/system/cpu/cpufreq/boost
        elif [ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
            # Intel uses inverted logic (0 = boost enabled)
            if [ "$CPU_BOOST" == "1" ]; then
                echo "0" > /sys/devices/system/cpu/intel_pstate/no_turbo
            else
                echo "1" > /sys/devices/system/cpu/intel_pstate/no_turbo
            fi
        fi
        echo -e "${GREEN}✓${RESET} CPU boost: $([ "$CPU_BOOST" == "1" ] && echo "enabled" || echo "disabled")"
    fi
}

# Function to apply memory optimizations
apply_memory_optimizations() {
    echo -e "\n${YELLOW}Applying memory optimizations...${RESET}"
    
    # Swappiness
    if [ ! -z "$VM_SWAPPINESS" ]; then
        sysctl -w vm.swappiness=$VM_SWAPPINESS > /dev/null
        echo -e "${GREEN}✓${RESET} VM swappiness set to: $VM_SWAPPINESS"
    fi
    
    # Dirty ratio
    if [ ! -z "$VM_DIRTY_RATIO" ]; then
        sysctl -w vm.dirty_ratio=$VM_DIRTY_RATIO > /dev/null
        sysctl -w vm.dirty_background_ratio=$((VM_DIRTY_RATIO / 4)) > /dev/null
        echo -e "${GREEN}✓${RESET} VM dirty ratio set to: $VM_DIRTY_RATIO"
    fi
    
    # ZRAM configuration
    if [ ! -z "$ZRAM_SIZE" ]; then
        # Update ZRAM config
        sed -i "s/PERCENT=.*/PERCENT=$ZRAM_SIZE/" /etc/default/zramswap 2>/dev/null || true
        
        # Restart ZRAM if running
        if systemctl is-active --quiet zramswap; then
            systemctl restart zramswap
            echo -e "${GREEN}✓${RESET} ZRAM size set to: ${ZRAM_SIZE}% of RAM"
        fi
    fi
    
    # Transparent Huge Pages
    if [ ! -z "$TRANSPARENT_HUGEPAGE" ]; then
        echo "$TRANSPARENT_HUGEPAGE" > /sys/kernel/mm/transparent_hugepage/enabled
        echo -e "${GREEN}✓${RESET} Transparent hugepages: $TRANSPARENT_HUGEPAGE"
    fi
}

# Function to apply GPU optimizations
apply_gpu_optimizations() {
    echo -e "\n${YELLOW}Applying GPU optimizations...${RESET}"
    
    # Check if NVIDIA GPU is present
    if command -v nvidia-smi &> /dev/null && nvidia-smi &> /dev/null; then
        # Persistence mode
        if [ ! -z "$NVIDIA_PERSISTENCE" ]; then
            nvidia-smi -pm $NVIDIA_PERSISTENCE &> /dev/null
            echo -e "${GREEN}✓${RESET} NVIDIA persistence mode: $([ "$NVIDIA_PERSISTENCE" == "1" ] && echo "enabled" || echo "disabled")"
        fi
        
        # Power limit
        if [ ! -z "$NVIDIA_POWER_LIMIT" ] && [ "$NVIDIA_POWER_LIMIT" != "default" ]; then
            nvidia-smi -pl $NVIDIA_POWER_LIMIT &> /dev/null || true
            echo -e "${GREEN}✓${RESET} NVIDIA power limit: ${NVIDIA_POWER_LIMIT}W"
        fi
        
        # Clock offset
        if [ ! -z "$NVIDIA_CLOCK_OFFSET" ] && [ "$NVIDIA_CLOCK_OFFSET" != "0" ]; then
            nvidia-settings -a "[gpu:0]/GPUGraphicsClockOffset[3]=$NVIDIA_CLOCK_OFFSET" &> /dev/null || true
            nvidia-settings -a "[gpu:0]/GPUMemoryTransferRateOffset[3]=$NVIDIA_CLOCK_OFFSET" &> /dev/null || true
            echo -e "${GREEN}✓${RESET} NVIDIA clock offset: +${NVIDIA_CLOCK_OFFSET}MHz"
        fi
        
        # Set compute mode
        nvidia-smi -c EXCLUSIVE_PROCESS &> /dev/null || true
        echo -e "${GREEN}✓${RESET} NVIDIA compute mode: exclusive process"
    else
        echo -e "${YELLOW}⚠${RESET} No NVIDIA GPU detected or drivers not loaded"
    fi
}

# Function to apply storage optimizations
apply_storage_optimizations() {
    echo -e "\n${YELLOW}Applying storage optimizations...${RESET}"
    
    # I/O Scheduler for NVMe
    if [ ! -z "$IO_SCHEDULER" ]; then
        for disk in /sys/block/nvme*/queue/scheduler; do
            if [ -f "$disk" ]; then
                echo "$IO_SCHEDULER" > $disk 2>/dev/null || true
            fi
        done
        echo -e "${GREEN}✓${RESET} NVMe I/O scheduler: $IO_SCHEDULER"
    fi
    
    # Read-ahead
    if [ ! -z "$READAHEAD_KB" ]; then
        for disk in /sys/block/nvme*/queue/read_ahead_kb; do
            if [ -f "$disk" ]; then
                echo "$READAHEAD_KB" > $disk 2>/dev/null || true
            fi
        done
        echo -e "${GREEN}✓${RESET} NVMe read-ahead: ${READAHEAD_KB}KB"
    fi
    
    # NVMe power saving
    for nvme in /sys/class/nvme/nvme*/power/control; do
        if [ -f "$nvme" ]; then
            echo "on" > $nvme 2>/dev/null || true
        fi
    done
    echo -e "${GREEN}✓${RESET} NVMe power saving: disabled"
}

# Function to apply tuned profile
apply_tuned_profile() {
    echo -e "\n${YELLOW}Applying tuned profile...${RESET}"
    
    if [ ! -z "$TUNED_PROFILE" ] && command -v tuned-adm &> /dev/null; then
        tuned-adm profile $TUNED_PROFILE
        echo -e "${GREEN}✓${RESET} Tuned profile: $TUNED_PROFILE"
    fi
}

# Function to apply kernel parameters
apply_kernel_parameters() {
    echo -e "\n${YELLOW}Applying kernel parameters...${RESET}"
    
    # Kernel preemption model (requires reboot to take effect)
    if [ ! -z "$KERNEL_PREEMPT" ]; then
        # Update GRUB configuration
        if grep -q "preempt=" /etc/default/grub; then
            sed -i "s/preempt=[^ ]*/preempt=$KERNEL_PREEMPT/" /etc/default/grub
        else
            sed -i "s/GRUB_CMDLINE_LINUX_DEFAULT=\"/GRUB_CMDLINE_LINUX_DEFAULT=\"preempt=$KERNEL_PREEMPT /" /etc/default/grub
        fi
        echo -e "${GREEN}✓${RESET} Kernel preemption: $KERNEL_PREEMPT (requires reboot)"
    fi
    
    # Apply sysctl optimizations based on profile
    case "$SYSTEM_PROFILE" in
        ultimate)
            sysctl -w kernel.sched_migration_cost_ns=500000 > /dev/null
            sysctl -w kernel.sched_min_granularity_ns=2000000 > /dev/null
            sysctl -w kernel.sched_wakeup_granularity_ns=3000000 > /dev/null
            echo -e "${GREEN}✓${RESET} Ultimate performance kernel scheduling"
            ;;
        gaming)
            sysctl -w kernel.sched_migration_cost_ns=1000000 > /dev/null
            sysctl -w kernel.sched_min_granularity_ns=4000000 > /dev/null
            sysctl -w kernel.sched_wakeup_granularity_ns=5000000 > /dev/null
            echo -e "${GREEN}✓${RESET} Gaming-optimized kernel scheduling"
            ;;
        server)
            sysctl -w kernel.sched_migration_cost_ns=5000000 > /dev/null
            sysctl -w kernel.sched_min_granularity_ns=10000000 > /dev/null
            sysctl -w kernel.sched_wakeup_granularity_ns=15000000 > /dev/null
            echo -e "${GREEN}✓${RESET} Server-optimized kernel scheduling"
            ;;
    esac
}

# Function to create systemd service for persistence
create_persistence_service() {
    echo -e "\n${YELLOW}Creating optimization persistence service...${RESET}"
    
    cat > /etc/systemd/system/hecate-optimizations.service << 'EOF'
[Unit]
Description=HecateOS Hardware Optimizations
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/hecate-apply-optimizations
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable hecate-optimizations.service
    echo -e "${GREEN}✓${RESET} Optimization service enabled for boot persistence"
}

# Main execution
main() {
    apply_cpu_optimizations
    apply_memory_optimizations
    apply_gpu_optimizations
    apply_storage_optimizations
    apply_tuned_profile
    apply_kernel_parameters
    
    # Create persistence service if not exists
    if [ ! -f /etc/systemd/system/hecate-optimizations.service ]; then
        create_persistence_service
    fi
    
    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${GREEN}All optimizations applied successfully!${RESET}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo ""
    echo -e "${CYAN}System profile: ${SYSTEM_PROFILE}${RESET}"
    echo -e "${CYAN}Some changes may require a reboot to take full effect${RESET}"
}

main "$@"