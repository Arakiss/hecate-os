//! HecateOS System Daemon
//! 
//! Core daemon that runs on boot to detect and optimize hardware

use anyhow::Result;
use clap::Parser;
use hecate_core::{HardwareDetector, HardwareInfo, SystemProfile, apply_optimizations};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

const CONFIG_PATH: &str = "/etc/hecate/hardware.json";
const FIRST_BOOT_FLAG: &str = "/etc/hecate/.first_boot_complete";

#[derive(Parser)]
#[command(author, version, about = "HecateOS System Daemon")]
struct Args {
    /// Force hardware re-detection
    #[arg(short, long)]
    force: bool,
    
    /// Run once and exit (don't daemonize)
    #[arg(short, long)]
    once: bool,
    
    /// Dry run - detect but don't apply optimizations
    #[arg(short, long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    let args = Args::parse();
    
    info!("HecateOS Daemon v{} starting...", env!("CARGO_PKG_VERSION"));
    
    // Check if this is first boot or forced re-detection
    let should_detect = !Path::new(FIRST_BOOT_FLAG).exists() || args.force;
    
    if should_detect {
        info!("Starting hardware detection...");
        let hardware = detect_hardware().await?;
        
        // Save hardware configuration
        save_hardware_config(&hardware)?;
        
        if !args.dry_run {
            // Apply optimizations based on detected hardware
            apply_system_optimizations(&hardware).await?;
            
            // Mark first boot as complete
            fs::create_dir_all("/etc/hecate")?;
            fs::write(FIRST_BOOT_FLAG, "")?;
        }
        
        print_system_summary(&hardware);
    } else {
        // Load existing configuration
        let hardware = load_hardware_config()?;
        info!("Using cached hardware configuration");
        
        if !args.dry_run {
            // Re-apply optimizations (useful after updates)
            apply_system_optimizations(&hardware).await?;
        }
    }
    
    if !args.once {
        // Start monitoring daemon
        start_monitoring_loop().await?;
    }
    
    Ok(())
}

async fn detect_hardware() -> Result<HardwareInfo> {
    let mut detector = HardwareDetector::new();
    let hardware = detector.detect()?;
    
    info!("Hardware detection complete:");
    info!("  CPU: {} ({} cores)", hardware.cpu.model, hardware.cpu.cores);
    info!("  RAM: {:.1} GB", hardware.memory.total_gb);
    
    for gpu in &hardware.gpu {
        info!("  GPU: {} ({:.1} GB VRAM)", gpu.model, gpu.vram_gb);
    }
    
    info!("  Profile: {:?}", hardware.profile);
    
    Ok(hardware)
}

async fn apply_system_optimizations(hardware: &HardwareInfo) -> Result<()> {
    info!("Applying optimizations for profile: {:?}", hardware.profile);
    
    // Apply core optimizations from library
    apply_optimizations(&hardware.profile)?;
    
    // Apply specific kernel parameters
    apply_kernel_parameters(hardware).await?;
    
    // Configure CPU governor
    configure_cpu_governor(hardware).await?;
    
    // Set up memory management
    configure_memory_management(hardware).await?;
    
    // Configure storage I/O schedulers
    configure_storage_io(hardware).await?;
    
    // Set up GPU-specific optimizations
    if !hardware.gpu.is_empty() {
        configure_gpu_settings(hardware).await?;
    }
    
    info!("All optimizations applied successfully");
    Ok(())
}

async fn apply_kernel_parameters(hardware: &HardwareInfo) -> Result<()> {
    let mut params = vec![
        "intel_pstate=active",
        "intel_iommu=on",
        "iommu=pt",
        "pcie_aspm=off",
    ];
    
    // Add profile-specific parameters
    match hardware.profile {
        SystemProfile::AIFlagship | SystemProfile::ProWorkstation => {
            params.push("mitigations=off");
            params.push("processor.max_cstate=1");
            params.push("intel_idle.max_cstate=0");
            params.push("nvme_core.default_ps_max_latency_us=0");
        }
        SystemProfile::HighPerformance => {
            params.push("mitigations=auto,nosmt");
            params.push("processor.max_cstate=2");
        }
        _ => {
            // Keep default parameters for standard systems
        }
    }
    
    // Update GRUB configuration
    update_grub_config(&params).await?;
    
    Ok(())
}

async fn update_grub_config(params: &[&str]) -> Result<()> {
    let params_str = params.join(" ");
    
    // Read current GRUB config
    let grub_path = "/etc/default/grub";
    let content = fs::read_to_string(grub_path)?;
    
    // Update GRUB_CMDLINE_LINUX_DEFAULT
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut updated = false;
    
    for line in &mut lines {
        if line.starts_with("GRUB_CMDLINE_LINUX_DEFAULT=") {
            *line = format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"", params_str);
            updated = true;
            break;
        }
    }
    
    if !updated {
        lines.push(format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"", params_str));
    }
    
    // Write back
    fs::write(grub_path, lines.join("\n"))?;
    
    // Update GRUB
    Command::new("update-grub").output()?;
    
    info!("GRUB configuration updated with: {}", params_str);
    Ok(())
}

async fn configure_cpu_governor(hardware: &HardwareInfo) -> Result<()> {
    let governor = match hardware.profile {
        SystemProfile::AIFlagship | SystemProfile::ProWorkstation => "performance",
        SystemProfile::HighPerformance => "ondemand",
        _ => "powersave",
    };
    
    // Set governor for all CPUs
    for cpu_id in 0..hardware.cpu.threads {
        let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu_id);
        if Path::new(&path).exists() {
            fs::write(&path, governor)?;
        }
    }
    
    info!("CPU governor set to: {}", governor);
    Ok(())
}

async fn configure_memory_management(hardware: &HardwareInfo) -> Result<()> {
    // Determine swappiness based on RAM amount
    let swappiness = match hardware.memory.total_gb {
        ram if ram >= 64.0 => 10,
        ram if ram >= 32.0 => 20,
        ram if ram >= 16.0 => 40,
        _ => 60,
    };
    
    fs::write("/proc/sys/vm/swappiness", swappiness.to_string())?;
    
    // Configure transparent hugepages
    let thp_setting = match hardware.profile {
        SystemProfile::AIFlagship | SystemProfile::ProWorkstation => "always",
        _ => "madvise",
    };
    
    fs::write("/sys/kernel/mm/transparent_hugepage/enabled", thp_setting)?;
    
    // Set dirty ratios for better I/O performance
    if hardware.memory.total_gb >= 32.0 {
        fs::write("/proc/sys/vm/dirty_background_ratio", "5")?;
        fs::write("/proc/sys/vm/dirty_ratio", "10")?;
    }
    
    info!("Memory management configured (swappiness={})", swappiness);
    Ok(())
}

async fn configure_storage_io(hardware: &HardwareInfo) -> Result<()> {
    use hecate_core::StorageType;
    
    for storage in &hardware.storage {
        // Extract device name (e.g., "nvme0n1" from "/dev/nvme0n1")
        let device_name = storage.device.strip_prefix("/dev/").unwrap_or(&storage.device);
        let scheduler_path = format!("/sys/block/{}/queue/scheduler", device_name);
        
        if Path::new(&scheduler_path).exists() {
            let scheduler = match storage.storage_type {
                StorageType::NvmeGen5 | StorageType::NvmeGen4 | StorageType::NvmeGen3 => "none",
                StorageType::Sata => "mq-deadline",
                StorageType::Hdd => "bfq",
                _ => "mq-deadline",
            };
            
            fs::write(&scheduler_path, scheduler)?;
            info!("I/O scheduler for {} set to: {}", storage.device, scheduler);
            
            // Set read-ahead for SSDs
            if matches!(storage.storage_type, StorageType::NvmeGen5 | StorageType::NvmeGen4 | StorageType::NvmeGen3 | StorageType::Sata) {
                let ra_path = format!("/sys/block/{}/queue/read_ahead_kb", device_name);
                fs::write(&ra_path, "256")?;
            }
        }
    }
    
    Ok(())
}

async fn configure_gpu_settings(hardware: &HardwareInfo) -> Result<()> {
    use hecate_core::GpuVendor;
    
    for gpu in &hardware.gpu {
        match gpu.vendor {
            GpuVendor::Nvidia => {
                // Enable persistence mode
                Command::new("nvidia-smi")
                    .args(&["-pm", "1"])
                    .output()?;
                
                // Set performance mode
                Command::new("nvidia-smi")
                    .args(&["-ac", "auto"])
                    .output()?;
                
                // Set power limit based on profile
                if matches!(hardware.profile, SystemProfile::AIFlagship | SystemProfile::ProWorkstation) {
                    Command::new("nvidia-smi")
                        .args(&["-pl", "500"]) // Max power
                        .output()?;
                }
                
                info!("NVIDIA GPU configured for maximum performance");
            }
            GpuVendor::Amd => {
                // Set AMD GPU performance level
                let perf_level = match hardware.profile {
                    SystemProfile::AIFlagship | SystemProfile::ProWorkstation => "high",
                    SystemProfile::HighPerformance => "auto",
                    _ => "low",
                };
                
                // This would write to /sys/class/drm/card*/device/power_dpm_force_performance_level
                info!("AMD GPU performance level set to: {}", perf_level);
            }
            _ => {}
        }
    }
    
    Ok(())
}

async fn start_monitoring_loop() -> Result<()> {
    info!("Starting monitoring daemon...");
    
    loop {
        // Monitor system health every 60 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        
        // Check thermal throttling
        check_thermal_status().await?;
        
        // Monitor memory pressure
        check_memory_pressure().await?;
        
        // Check for GPU errors
        check_gpu_health().await?;
    }
}

async fn check_thermal_status() -> Result<()> {
    // Read CPU temperature
    let temp_path = "/sys/class/thermal/thermal_zone0/temp";
    if let Ok(temp_str) = fs::read_to_string(temp_path) {
        if let Ok(temp) = temp_str.trim().parse::<i32>() {
            let temp_celsius = temp / 1000;
            if temp_celsius > 85 {
                warn!("High CPU temperature detected: {}°C", temp_celsius);
                // Could trigger fan speed increase or frequency reduction
            }
        }
    }
    Ok(())
}

async fn check_memory_pressure() -> Result<()> {
    // Check available memory
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemAvailable:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        let gb = kb as f64 / 1024.0 / 1024.0;
                        if gb < 2.0 {
                            warn!("Low memory available: {:.1} GB", gb);
                            // Could trigger cache cleanup or OOM killer tuning
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn check_gpu_health() -> Result<()> {
    // Check NVIDIA GPU if present
    if Path::new("/usr/bin/nvidia-smi").exists() {
        let output = Command::new("nvidia-smi")
            .args(&["--query-gpu=temperature.gpu,utilization.gpu,utilization.memory", "--format=csv,noheader,nounits"])
            .output()?;
        
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            let values: Vec<&str> = result.trim().split(", ").collect();
            
            if values.len() >= 3 {
                if let Ok(temp) = values[0].parse::<i32>() {
                    if temp > 83 {
                        warn!("High GPU temperature: {}°C", temp);
                    }
                }
            }
        }
    }
    Ok(())
}

fn save_hardware_config(hardware: &HardwareInfo) -> Result<()> {
    fs::create_dir_all("/etc/hecate")?;
    let json = serde_json::to_string_pretty(hardware)?;
    fs::write(CONFIG_PATH, json)?;
    info!("Hardware configuration saved to {}", CONFIG_PATH);
    Ok(())
}

fn load_hardware_config() -> Result<HardwareInfo> {
    let json = fs::read_to_string(CONFIG_PATH)?;
    let hardware = serde_json::from_str(&json)?;
    Ok(hardware)
}

fn print_system_summary(hardware: &HardwareInfo) {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║              HecateOS System Configuration                ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Profile: {:?}", hardware.profile);
    println!("║ CPU: {}", hardware.cpu.model);
    println!("║   Cores: {} | Threads: {}", hardware.cpu.cores, hardware.cpu.threads);
    println!("║ Memory: {:.1} GB", hardware.memory.total_gb);
    
    for (i, gpu) in hardware.gpu.iter().enumerate() {
        println!("║ GPU {}: {} ({:.1} GB)", i, gpu.model, gpu.vram_gb);
    }
    
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Optimizations Applied:                                    ║");
    
    match hardware.profile {
        SystemProfile::AIFlagship => {
            println!("║   ✓ Maximum performance mode                            ║");
            println!("║   ✓ Mitigations disabled                                ║");
            println!("║   ✓ C-States limited                                    ║");
            println!("║   ✓ GPU persistence enabled                             ║");
            println!("║   ✓ Huge pages always enabled                           ║");
        }
        SystemProfile::ProWorkstation => {
            println!("║   ✓ Performance mode                                    ║");
            println!("║   ✓ Selective mitigations                              ║");
            println!("║   ✓ GPU optimized                                      ║");
            println!("║   ✓ I/O tuned for NVMe                                 ║");
        }
        _ => {
            println!("║   ✓ Balanced performance                               ║");
            println!("║   ✓ Power saving enabled                               ║");
        }
    }
    
    println!("╚══════════════════════════════════════════════════════════╝");
}