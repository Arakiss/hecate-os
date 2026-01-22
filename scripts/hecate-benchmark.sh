#!/bin/bash
#
# HecateOS Performance Benchmarking Suite
# Comprehensive system performance testing
#

set -e

# Colors
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
WHITE='\033[1;37m'
RESET='\033[0m'

# Benchmark results file
RESULTS_DIR="/var/log/hecate/benchmarks"
RESULTS_FILE="$RESULTS_DIR/benchmark-$(date +%Y%m%d-%H%M%S).json"
COMPARISON_FILE="/etc/hecate/baseline-benchmarks.json"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Initialize results
RESULTS="{}"

# Show banner
show_banner() {
    echo -e "${PURPLE}"
    cat << 'EOF'
╦ ╦┌─┐┌─┐┌─┐┌┬┐┌─┐╔═╗╔═╗
╠═╣├┤ │  ├─┤ │ ├┤ ║ ║╚═╗
╩ ╩└─┘└─┘┴ ┴ ┴ └─┘╚═╝╚═╝

Performance Benchmark Suite
EOF
    echo -e "${RESET}"
}

# Progress bar
progress_bar() {
    local current=$1
    local total=$2
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((current * width / total))
    
    printf "\r["
    printf "%${filled}s" | tr ' ' '█'
    printf "%$((width - filled))s" | tr ' ' '░'
    printf "] %d%%" "$percentage"
}

# CPU Benchmark
benchmark_cpu() {
    echo -e "\n${CYAN}Running CPU Benchmark...${RESET}"
    
    # Single-thread performance
    echo -n "  Single-thread performance: "
    SINGLE_SCORE=$(sysbench cpu --cpu-max-prime=20000 --threads=1 run 2>/dev/null | \
                   grep "events per second" | awk '{print $4}')
    echo -e "${GREEN}$SINGLE_SCORE events/sec${RESET}"
    
    # Multi-thread performance
    THREADS=$(nproc)
    echo -n "  Multi-thread performance ($THREADS threads): "
    MULTI_SCORE=$(sysbench cpu --cpu-max-prime=20000 --threads=$THREADS run 2>/dev/null | \
                  grep "events per second" | awk '{print $4}')
    echo -e "${GREEN}$MULTI_SCORE events/sec${RESET}"
    
    # Floating point operations
    echo -n "  Floating point operations: "
    FLOAT_OPS=$(echo "scale=2; 1000000000 / $(echo 'for(i=0;i<1000000;i++) sqrt(i)' | \
                time -p bc 2>&1 | grep real | awk '{print $2}')" | bc 2>/dev/null || echo "N/A")
    echo -e "${GREEN}$FLOAT_OPS MFLOPS${RESET}"
    
    # Add to results
    RESULTS=$(echo "$RESULTS" | jq ". + {
        \"cpu\": {
            \"single_thread\": $SINGLE_SCORE,
            \"multi_thread\": $MULTI_SCORE,
            \"threads\": $THREADS,
            \"float_ops\": \"$FLOAT_OPS\"
        }
    }")
}

# Memory Benchmark
benchmark_memory() {
    echo -e "\n${CYAN}Running Memory Benchmark...${RESET}"
    
    # Memory bandwidth
    echo -n "  Memory read bandwidth: "
    MEM_READ=$(sysbench memory --memory-oper=read --memory-access-mode=seq run 2>/dev/null | \
               grep "transferred" | sed 's/.*(\(.*\))/\1/')
    echo -e "${GREEN}$MEM_READ${RESET}"
    
    echo -n "  Memory write bandwidth: "
    MEM_WRITE=$(sysbench memory --memory-oper=write --memory-access-mode=seq run 2>/dev/null | \
                grep "transferred" | sed 's/.*(\(.*\))/\1/')
    echo -e "${GREEN}$MEM_WRITE${RESET}"
    
    # Memory latency
    echo -n "  Memory latency: "
    MEM_LATENCY=$(sysbench memory --memory-oper=read --memory-access-mode=rnd run 2>/dev/null | \
                  grep "total time:" | awk '{print $3}' | sed 's/s//')
    echo -e "${GREEN}${MEM_LATENCY}s${RESET}"
    
    RESULTS=$(echo "$RESULTS" | jq ". + {
        \"memory\": {
            \"read_bandwidth\": \"$MEM_READ\",
            \"write_bandwidth\": \"$MEM_WRITE\",
            \"latency\": \"$MEM_LATENCY\"
        }
    }")
}

# Storage Benchmark
benchmark_storage() {
    echo -e "\n${CYAN}Running Storage Benchmark...${RESET}"
    
    TEST_FILE="/tmp/hecate-benchmark-test"
    TEST_SIZE="1G"
    
    # Sequential read
    echo -n "  Sequential read: "
    SEQ_READ=$(dd if=/dev/zero of=$TEST_FILE bs=1M count=1024 conv=fdatasync 2>&1 | \
               grep -o '[0-9.]* [MG]B/s' | tail -1)
    echo -e "${GREEN}$SEQ_READ${RESET}"
    
    # Sequential write
    echo -n "  Sequential write: "
    SEQ_WRITE=$(dd if=$TEST_FILE of=/dev/null bs=1M 2>&1 | \
                grep -o '[0-9.]* [MG]B/s' | tail -1)
    echo -e "${GREEN}$SEQ_WRITE${RESET}"
    
    # Random 4K IOPS
    if command -v fio &> /dev/null; then
        echo -n "  Random 4K read IOPS: "
        RAND_READ=$(fio --name=rand-read --ioengine=libaio --rw=randread --bs=4k \
                    --direct=1 --size=256M --numjobs=1 --runtime=10 --time_based \
                    --filename=$TEST_FILE 2>/dev/null | grep "IOPS=" | \
                    head -1 | grep -o 'IOPS=[0-9.]*[km]*' | cut -d= -f2)
        echo -e "${GREEN}$RAND_READ${RESET}"
        
        echo -n "  Random 4K write IOPS: "
        RAND_WRITE=$(fio --name=rand-write --ioengine=libaio --rw=randwrite --bs=4k \
                     --direct=1 --size=256M --numjobs=1 --runtime=10 --time_based \
                     --filename=$TEST_FILE 2>/dev/null | grep "IOPS=" | \
                     head -1 | grep -o 'IOPS=[0-9.]*[km]*' | cut -d= -f2)
        echo -e "${GREEN}$RAND_WRITE${RESET}"
    else
        RAND_READ="N/A"
        RAND_WRITE="N/A"
    fi
    
    # Cleanup
    rm -f $TEST_FILE
    
    RESULTS=$(echo "$RESULTS" | jq ". + {
        \"storage\": {
            \"sequential_read\": \"$SEQ_READ\",
            \"sequential_write\": \"$SEQ_WRITE\",
            \"random_4k_read_iops\": \"$RAND_READ\",
            \"random_4k_write_iops\": \"$RAND_WRITE\"
        }
    }")
}

# GPU Benchmark (NVIDIA)
benchmark_gpu() {
    echo -e "\n${CYAN}Running GPU Benchmark...${RESET}"
    
    if ! command -v nvidia-smi &> /dev/null; then
        echo -e "${YELLOW}  No NVIDIA GPU detected${RESET}"
        RESULTS=$(echo "$RESULTS" | jq '. + {"gpu": {"available": false}}')
        return
    fi
    
    # GPU info
    GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
    GPU_MEMORY=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)
    
    echo "  GPU: $GPU_NAME ($GPU_MEMORY MB)"
    
    # CUDA bandwidth test
    if [ -f /usr/local/cuda/samples/1_Utilities/bandwidthTest/bandwidthTest ]; then
        echo -n "  Device to Device Bandwidth: "
        D2D_BW=$(/usr/local/cuda/samples/1_Utilities/bandwidthTest/bandwidthTest --mode=shmoo --csv 2>/dev/null | \
                 tail -1 | cut -d, -f2)
        echo -e "${GREEN}${D2D_BW} GB/s${RESET}"
        
        echo -n "  Host to Device Bandwidth: "
        H2D_BW=$(/usr/local/cuda/samples/1_Utilities/bandwidthTest/bandwidthTest --mode=pinned --csv 2>/dev/null | \
                 grep "Host to Device" | cut -d, -f3)
        echo -e "${GREEN}${H2D_BW} GB/s${RESET}"
    else
        D2D_BW="N/A"
        H2D_BW="N/A"
    fi
    
    # Simple CUDA compute test
    if command -v python3 &> /dev/null && python3 -c "import torch" 2>/dev/null; then
        echo -n "  CUDA TFLOPS (FP32): "
        TFLOPS=$(python3 -c "
import torch
import time
size = 4096
a = torch.rand(size, size).cuda()
b = torch.rand(size, size).cuda()
torch.cuda.synchronize()
start = time.time()
for _ in range(100):
    c = torch.matmul(a, b)
torch.cuda.synchronize()
elapsed = time.time() - start
ops = 2.0 * size ** 3 * 100
tflops = ops / elapsed / 1e12
print(f'{tflops:.2f}')
" 2>/dev/null || echo "N/A")
        echo -e "${GREEN}${TFLOPS} TFLOPS${RESET}"
    else
        TFLOPS="N/A"
    fi
    
    RESULTS=$(echo "$RESULTS" | jq ". + {
        \"gpu\": {
            \"available\": true,
            \"name\": \"$GPU_NAME\",
            \"memory_mb\": $GPU_MEMORY,
            \"device_bandwidth_gbps\": \"$D2D_BW\",
            \"host_bandwidth_gbps\": \"$H2D_BW\",
            \"compute_tflops\": \"$TFLOPS\"
        }
    }")
}

# Network Benchmark
benchmark_network() {
    echo -e "\n${CYAN}Running Network Benchmark...${RESET}"
    
    # Loopback bandwidth
    echo -n "  Loopback bandwidth: "
    if command -v iperf3 &> /dev/null; then
        # Start server in background
        iperf3 -s -D -p 5201 &> /dev/null
        sleep 1
        
        LOOPBACK_BW=$(iperf3 -c 127.0.0.1 -p 5201 -t 5 -f g 2>/dev/null | \
                      grep "sender" | awk '{print $(NF-2), $(NF-1)}')
        
        # Kill server
        pkill iperf3 2>/dev/null || true
        
        echo -e "${GREEN}$LOOPBACK_BW${RESET}"
    else
        LOOPBACK_BW="N/A"
        echo -e "${YELLOW}N/A (iperf3 not installed)${RESET}"
    fi
    
    # Network latency to common servers
    echo -n "  Network latency (8.8.8.8): "
    if ping -c 4 8.8.8.8 &> /dev/null; then
        LATENCY=$(ping -c 4 8.8.8.8 | grep "rtt" | cut -d'/' -f5)
        echo -e "${GREEN}${LATENCY}ms${RESET}"
    else
        LATENCY="N/A"
        echo -e "${YELLOW}N/A${RESET}"
    fi
    
    RESULTS=$(echo "$RESULTS" | jq ". + {
        \"network\": {
            \"loopback_bandwidth\": \"$LOOPBACK_BW\",
            \"external_latency_ms\": \"$LATENCY\"
        }
    }")
}

# System info
gather_system_info() {
    echo -e "\n${CYAN}Gathering System Information...${RESET}"
    
    KERNEL=$(uname -r)
    CPU_MODEL=$(lscpu | grep "Model name" | cut -d: -f2 | xargs)
    TOTAL_MEM=$(free -h | grep Mem | awk '{print $2}')
    PROFILE=$(jq -r '.system.profile' /etc/hecate/hardware-profile.json 2>/dev/null || echo "unknown")
    
    echo "  Kernel: $KERNEL"
    echo "  CPU: $CPU_MODEL"
    echo "  Memory: $TOTAL_MEM"
    echo "  Profile: $PROFILE"
    
    RESULTS=$(echo "$RESULTS" | jq ". + {
        \"system\": {
            \"kernel\": \"$KERNEL\",
            \"cpu_model\": \"$CPU_MODEL\",
            \"total_memory\": \"$TOTAL_MEM\",
            \"profile\": \"$PROFILE\",
            \"timestamp\": \"$(date -Iseconds)\"
        }
    }")
}

# Compare with baseline
compare_results() {
    echo -e "\n${CYAN}Performance Analysis...${RESET}"
    
    if [ ! -f "$COMPARISON_FILE" ]; then
        echo -e "${YELLOW}  No baseline found. Current results will be saved as baseline.${RESET}"
        cp "$RESULTS_FILE" "$COMPARISON_FILE"
        return
    fi
    
    # Compare CPU scores
    BASELINE_CPU=$(jq -r '.cpu.multi_thread' "$COMPARISON_FILE" 2>/dev/null || echo "0")
    CURRENT_CPU=$(echo "$RESULTS" | jq -r '.cpu.multi_thread')
    
    if [ "$BASELINE_CPU" != "0" ] && [ "$CURRENT_CPU" != "null" ]; then
        DIFF=$(echo "scale=2; (($CURRENT_CPU - $BASELINE_CPU) / $BASELINE_CPU) * 100" | bc)
        
        if (( $(echo "$DIFF > 0" | bc -l) )); then
            echo -e "  CPU Performance: ${GREEN}+${DIFF}%${RESET} vs baseline"
        elif (( $(echo "$DIFF < 0" | bc -l) )); then
            echo -e "  CPU Performance: ${RED}${DIFF}%${RESET} vs baseline"
        else
            echo -e "  CPU Performance: ${YELLOW}No change${RESET}"
        fi
    fi
    
    # Overall score
    echo -e "\n${WHITE}Overall Performance Score:${RESET}"
    
    # Calculate simple score (normalized to 100)
    SCORE=0
    COUNT=0
    
    # CPU contribution
    if [ "$CURRENT_CPU" != "null" ]; then
        CPU_NORM=$(echo "scale=2; ($CURRENT_CPU / 10000) * 25" | bc)
        SCORE=$(echo "$SCORE + $CPU_NORM" | bc)
        COUNT=$((COUNT + 1))
    fi
    
    # Add more scoring logic as needed...
    
    # Display score with visual indicator
    FINAL_SCORE=$(echo "scale=0; $SCORE / 1" | bc)
    
    if [ "$FINAL_SCORE" -ge 90 ]; then
        COLOR=$GREEN
        RATING="EXCELLENT"
    elif [ "$FINAL_SCORE" -ge 70 ]; then
        COLOR=$CYAN
        RATING="GOOD"
    elif [ "$FINAL_SCORE" -ge 50 ]; then
        COLOR=$YELLOW
        RATING="AVERAGE"
    else
        COLOR=$RED
        RATING="NEEDS OPTIMIZATION"
    fi
    
    echo -e "  ${COLOR}████████████████████${RESET} $FINAL_SCORE/100 - $RATING"
}

# Export results
export_results() {
    echo "$RESULTS" > "$RESULTS_FILE"
    
    # Also create CSV for easy analysis
    CSV_FILE="$RESULTS_DIR/benchmark-$(date +%Y%m%d-%H%M%S).csv"
    
    echo "Metric,Value" > "$CSV_FILE"
    echo "$RESULTS" | jq -r '
        paths(scalars) as $p |
        $p | join(".") + "," + (getpath($p) | tostring)
    ' >> "$CSV_FILE"
    
    echo -e "\n${GREEN}Results saved to:${RESET}"
    echo "  JSON: $RESULTS_FILE"
    echo "  CSV: $CSV_FILE"
}

# Main execution
main() {
    show_banner
    
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        echo -e "${YELLOW}Note: Some benchmarks may require root privileges for accuracy${RESET}"
    fi
    
    # Check for required tools
    MISSING_TOOLS=""
    for tool in sysbench jq bc; do
        if ! command -v $tool &> /dev/null; then
            MISSING_TOOLS="$MISSING_TOOLS $tool"
        fi
    done
    
    if [ ! -z "$MISSING_TOOLS" ]; then
        echo -e "${YELLOW}Installing missing tools:$MISSING_TOOLS${RESET}"
        sudo apt-get update && sudo apt-get install -y $MISSING_TOOLS
    fi
    
    # Run benchmarks
    gather_system_info
    
    TOTAL_TESTS=5
    CURRENT_TEST=0
    
    echo -e "\n${WHITE}Running Benchmarks...${RESET}"
    
    progress_bar $((++CURRENT_TEST)) $TOTAL_TESTS
    benchmark_cpu
    
    progress_bar $((++CURRENT_TEST)) $TOTAL_TESTS
    benchmark_memory
    
    progress_bar $((++CURRENT_TEST)) $TOTAL_TESTS
    benchmark_storage
    
    progress_bar $((++CURRENT_TEST)) $TOTAL_TESTS
    benchmark_gpu
    
    progress_bar $((++CURRENT_TEST)) $TOTAL_TESTS
    benchmark_network
    
    echo -e "\n"
    
    # Save and analyze results
    export_results
    compare_results
    
    echo -e "\n${GREEN}Benchmark complete!${RESET}"
}

# Parse arguments
case "$1" in
    --quick)
        # Quick benchmark (skip intensive tests)
        QUICK_MODE=1
        main
        ;;
    --compare)
        # Compare two benchmark files
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo "Usage: $0 --compare <file1> <file2>"
            exit 1
        fi
        # TODO: Implement comparison
        ;;
    --help)
        echo "Usage: $0 [options]"
        echo "Options:"
        echo "  --quick     Run quick benchmark (skip intensive tests)"
        echo "  --compare   Compare two benchmark results"
        echo "  --help      Show this help"
        ;;
    *)
        main
        ;;
esac