//! HecateOS ISO Builder
//! 
//! A tool to customize Ubuntu ISOs with HecateOS components

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use walkdir::WalkDir;

mod iso;
mod config;
mod injector;

use config::HecateConfig;
use iso::IsoManager;
use injector::ComponentInjector;

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
        /// Input Ubuntu ISO file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output ISO file
        #[arg(short, long, default_value = "hecateos.iso")]
        output: PathBuf,
        
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
        
        /// Include compiled Rust binaries
        #[arg(long)]
        with_binaries: bool,
        
        /// Include source code
        #[arg(long)]
        with_source: bool,
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
        Commands::Build { input, output, config, with_binaries, with_source } => {
            build_iso(input, output, config, with_binaries, with_source).await?;
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

async fn build_iso(
    input: PathBuf, 
    output: PathBuf, 
    config_path: Option<PathBuf>,
    with_binaries: bool,
    with_source: bool,
) -> Result<()> {
    println!("{}", "HecateOS ISO Builder".bright_cyan().bold());
    println!("{}", "=".repeat(40).bright_cyan());
    
    // Load configuration
    let config = if let Some(path) = config_path {
        HecateConfig::from_file(&path)?
    } else {
        HecateConfig::default()
    };
    
    // Verify input ISO exists
    if !input.exists() {
        eprintln!("{} Input ISO not found: {}", "Error:".red(), input.display());
        eprintln!("\nDownload Ubuntu 24.04 from:");
        eprintln!("  https://releases.ubuntu.com/24.04/");
        return Err(anyhow::anyhow!("Input ISO not found"));
    }
    
    // Create temporary working directory
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let work_dir = temp_dir.path();
    
    println!("📦 Extracting ISO...");
    let pb = create_progress_bar(100);
    
    // Extract ISO
    let iso_manager = IsoManager::new();
    let extract_dir = work_dir.join("iso");
    iso_manager.extract(&input, &extract_dir, &pb).await?;
    
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