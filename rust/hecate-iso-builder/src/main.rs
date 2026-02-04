//! HecateOS ISO Builder
//! 
//! A tool to customize Ubuntu ISOs with HecateOS components

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

mod iso;
mod iso_native;
mod iso_extractor;
mod config;
mod injector;
mod downloader;

use config::HecateConfig;
use iso::IsoManager;
use injector::ComponentInjector;
use downloader::IsoDownloader;

#[derive(Parser)]
#[command(name = "hecate-iso")]
#[command(about = "HecateOS ISO Builder - Customize Ubuntu ISOs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a customized HecateOS ISO
    Build {
        /// Download Ubuntu ISO automatically (24.04 or 22.04)
        #[arg(short = 'd', long)]
        download: Option<String>,
        /// Input Ubuntu ISO file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output ISO file
        #[arg(short, long, default_value = "hecateos.iso")]
        output: PathBuf,
        
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
        
        /// Include compiled Rust binaries (builds them if needed)
        #[arg(long)]
        with_binaries: bool,
        
        /// Include source code
        #[arg(long)]
        with_source: bool,
        
        /// Skip building components (use existing binaries)
        #[arg(long)]
        skip_build: bool,
    },
    
    /// Extract an ISO for manual customization
    Extract {
        /// ISO file to extract
        iso: PathBuf,
        
        /// Output directory
        #[arg(short, long, default_value = "./iso-extract")]
        output: PathBuf,
    },
    
    /// Repack a modified ISO directory
    Repack {
        /// Directory containing ISO contents
        dir: PathBuf,
        
        /// Output ISO file
        #[arg(short, long, default_value = "custom.iso")]
        output: PathBuf,
        
        /// Volume label
        #[arg(short, long, default_value = "HECATEOS")]
        label: String,
    },
    
    /// Create a configuration template
    Init {
        /// Output config file
        #[arg(short, long, default_value = "hecate-iso.toml")]
        output: PathBuf,
    },
    
    /// Verify ISO integrity and HecateOS components
    Verify {
        /// ISO file to verify
        iso: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build { download, input, output, config, with_binaries, with_source, skip_build } => {
            build_iso(download, input, output, config, with_binaries, with_source, skip_build).await?;
        }
        Commands::Extract { iso, output } => {
            extract_iso(iso, output).await?;
        }
        Commands::Repack { dir, output, label } => {
            repack_iso(dir, output, label).await?;
        }
        Commands::Init { output } => {
            create_config_template(output)?;
        }
        Commands::Verify { iso } => {
            verify_iso(iso).await?;
        }
    }
    
    Ok(())
}

fn find_rust_project_root() -> Result<PathBuf> {
    // First check if we're already in the rust directory
    let current = std::env::current_dir()?;
    if current.join("Makefile").exists() && current.join("hecate-daemon").is_dir() {
        return Ok(current);
    }
    
    // Check if HECATE_ROOT env var is set
    if let Ok(root) = std::env::var("HECATE_ROOT") {
        let root_path = PathBuf::from(root);
        if root_path.join("Makefile").exists() && root_path.join("hecate-daemon").is_dir() {
            return Ok(root_path);
        }
    }
    
    // Try to find based on the executable location
    if let Ok(exe_path) = std::env::current_exe() {
        // Check if we're in target/release or target/debug
        if let Some(parent) = exe_path.parent() {
            if let Some(target) = parent.parent() {
                if let Some(rust_dir) = target.parent() {
                    if rust_dir.join("Makefile").exists() && rust_dir.join("hecate-daemon").is_dir() {
                        return Ok(rust_dir.to_path_buf());
                    }
                }
            }
        }
    }
    
    // Try searching upward from current directory
    let mut search_dir = current.clone();
    for _ in 0..5 {
        if search_dir.join("rust/Makefile").exists() && search_dir.join("rust/hecate-daemon").is_dir() {
            return Ok(search_dir.join("rust"));
        }
        if let Some(parent) = search_dir.parent() {
            search_dir = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    Err(anyhow::anyhow!(
        "Could not find HecateOS project root. Set HECATE_ROOT environment variable to /path/to/hecate-os/rust"
    ))
}

fn build_all_components() -> Result<()> {
    println!("🔨 Building all HecateOS components...");
    
    // Find the rust directory by looking for Makefile
    let rust_dir = find_rust_project_root()?;
    
    // List of components to build
    let components = vec![
        "hecate-daemon",
        "hecate-monitor", 
        "hecate-bench",
        "hecate-pkg",
        "hecate-gpu",
        "hecate-ml",
        "hecate-dev",
        "hecate-sign",
    ];
    
    let total = components.len();
    for (idx, component) in components.iter().enumerate() {
        println!("  [{}/{}] Building {}...", idx + 1, total, component);
        
        let component_dir = rust_dir.join(component);
        if !component_dir.exists() {
            eprintln!("    ⚠️  Component directory not found: {}", component);
            continue;
        }
        
        // Special handling for hecate-pkg which needs DATABASE_URL
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&component_dir)
            .arg("build")
            .arg("--release");
            
        if *component == "hecate-pkg" {
            cmd.env("DATABASE_URL", "sqlite:hecate-pkg.db");
        }
        
        let output = cmd.output()
            .context(format!("Failed to build {}", component))?;
        
        if !output.status.success() {
            eprintln!("    ❌ Build failed for {}", component);
            eprintln!("    Error: {}", String::from_utf8_lossy(&output.stderr));
            // Continue with other components instead of failing
        } else {
            println!("    ✅ {} built successfully", component);
        }
    }
    
    println!("✅ Component build complete");
    Ok(())
}

async fn build_iso(
    download: Option<String>,
    input: PathBuf, 
    output: PathBuf, 
    config_path: Option<PathBuf>,
    with_binaries: bool,
    with_source: bool,
    skip_build: bool,
) -> Result<()> {
    println!("{}", "HecateOS ISO Builder".bright_cyan().bold());
    println!("{}", "=".repeat(40).bright_cyan());
    
    // Build components first if needed
    if with_binaries && !skip_build {
        build_all_components()?;
    }
    
    // Load configuration
    let config = if let Some(path) = config_path {
        HecateConfig::from_file(&path)?
    } else {
        HecateConfig::default()
    };
    
    // Handle ISO download or use existing
    let iso_path = if let Some(ref version) = download {
        let download_path = PathBuf::from(format!("ubuntu-{}.iso", version));
        if !download_path.exists() {
            IsoDownloader::download_ubuntu(&version, &download_path).await?;
        } else {
            println!("ℹ️  Using cached ISO: {}", download_path.display());
        }
        download_path
    } else {
        // Verify input ISO exists
        if !input.exists() {
            eprintln!("{} Input ISO not found: {}", "Error:".red(), input.display());
            eprintln!("\nTry using --download option:");
            eprintln!("  hecate-iso build --download 24.04 -o hecateos.iso");
            return Err(anyhow::anyhow!("Input ISO not found"));
        }
        input
    };
    
    // Create temporary working directory
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let work_dir = temp_dir.path();
    
    println!("📦 Extracting ISO...");
    let pb = create_progress_bar(100);
    
    // Extract ISO
    let iso_manager = IsoManager::new();
    let extract_dir = work_dir.join("iso");
    iso_manager.extract(&iso_path, &extract_dir, &pb).await?;
    
    pb.finish_with_message("ISO extracted");
    
    // Inject HecateOS components
    println!("🔧 Injecting HecateOS components...");
    let injector = ComponentInjector::new(config);
    
    if with_binaries {
        println!("  Adding compiled binaries...");
        injector.inject_binaries(&extract_dir)?;
    }
    
    if with_source {
        println!("  Adding source code...");
        injector.inject_source(&extract_dir)?;
    }
    
    // Add configuration files
    println!("  Adding configuration...");
    injector.inject_config(&extract_dir)?;
    
    // Add installer scripts
    println!("  Adding installer scripts...");
    injector.inject_scripts(&extract_dir)?;
    
    // Modify boot configuration
    println!("  Modifying boot configuration...");
    injector.modify_boot_config(&extract_dir)?;
    
    // Repack ISO
    println!("📀 Creating new ISO...");
    let pb = create_progress_bar(100);
    iso_manager.repack(&extract_dir, &output, "HECATEOS", &pb).await?;
    pb.finish_with_message("ISO created");
    
    // Show summary
    let size = fs::metadata(&output)?.len() / 1_000_000;
    println!("\n{}", "✅ Build complete!".green().bold());
    println!("  Output: {}", output.display().to_string().bright_yellow());
    println!("  Size: {} MB", size);
    
    println!("\n{}", "Next steps:".bright_cyan());
    println!("  1. Test in VM: qemu-system-x86_64 -m 4G -cdrom {}", output.display());
    println!("  2. Or use VirtualBox/VMware");
    println!("  3. Or write to USB: sudo dd if={} of=/dev/sdX bs=4M", output.display());
    
    // Cleanup downloaded ISO if requested
    if download.is_some() && iso_path.exists() {
        println!("\n🧹 Cleaning up downloaded ISO...");
        IsoDownloader::cleanup(&iso_path)?;
    }
    
    Ok(())
}

async fn extract_iso(iso: PathBuf, output: PathBuf) -> Result<()> {
    println!("Extracting ISO to {}...", output.display());
    
    let pb = create_progress_bar(100);
    let iso_manager = IsoManager::new();
    iso_manager.extract(&iso, &output, &pb).await?;
    pb.finish_with_message("Extraction complete");
    
    println!("\n✅ ISO extracted to: {}", output.display());
    println!("\nYou can now modify the contents and repack with:");
    println!("  hecate-iso repack {} -o custom.iso", output.display());
    
    Ok(())
}

async fn repack_iso(dir: PathBuf, output: PathBuf, label: String) -> Result<()> {
    println!("Repacking ISO from {}...", dir.display());
    
    if !dir.exists() {
        return Err(anyhow::anyhow!("Directory not found: {}", dir.display()));
    }
    
    let pb = create_progress_bar(100);
    let iso_manager = IsoManager::new();
    iso_manager.repack(&dir, &output, &label, &pb).await?;
    pb.finish_with_message("ISO created");
    
    let size = fs::metadata(&output)?.len() / 1_000_000;
    println!("\n✅ ISO created: {} ({} MB)", output.display(), size);
    
    Ok(())
}

fn create_config_template(output: PathBuf) -> Result<()> {
    let config = HecateConfig::default();
    let toml = toml::to_string_pretty(&config)?;
    fs::write(&output, toml)?;
    
    println!("✅ Configuration template created: {}", output.display());
    println!("\nEdit this file to customize your ISO build.");
    
    Ok(())
}

async fn verify_iso(iso: PathBuf) -> Result<()> {
    println!("Verifying ISO: {}...", iso.display());
    
    let temp_dir = TempDir::new()?;
    let extract_dir = temp_dir.path().join("verify");
    
    // Extract ISO
    let pb = create_progress_bar(100);
    let iso_manager = IsoManager::new();
    iso_manager.extract(&iso, &extract_dir, &pb).await?;
    pb.finish();
    
    println!("\n{}", "ISO Contents:".bright_cyan());
    
    // Check for HecateOS components
    let hecate_dir = extract_dir.join("hecateos");
    if hecate_dir.exists() {
        println!("  ✅ HecateOS directory found");
        
        // Check binaries
        let bin_dir = hecate_dir.join("bin");
        if bin_dir.exists() {
            println!("  ✅ Binaries directory found:");
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let name = entry.file_name();
                let size = entry.metadata()?.len() / 1024;
                println!("      {} ({} KB)", name.to_string_lossy(), size);
            }
        }
        
        // Check scripts
        if hecate_dir.join("install.sh").exists() {
            println!("  ✅ Install script found");
        }
        
        // Check config
        if hecate_dir.join("config").exists() {
            println!("  ✅ Configuration directory found");
        }
    } else {
        println!("  ⚠️  No HecateOS components found");
        println!("  This appears to be a standard Ubuntu ISO");
    }
    
    // Check boot configuration
    let grub_cfg = extract_dir.join("boot/grub/grub.cfg");
    if grub_cfg.exists() {
        let content = fs::read_to_string(&grub_cfg)?;
        if content.contains("HecateOS") {
            println!("  ✅ Boot menu customized for HecateOS");
        } else {
            println!("  ℹ️  Standard boot menu");
        }
    }
    
    Ok(())
}

fn create_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb
}