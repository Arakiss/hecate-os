use anyhow::{Context, Result};
use std::process::Command;
use std::path::PathBuf;
use tracing::{info, warn};
use colored::*;

/// Create a standalone HecateOS ISO without Ubuntu base
pub async fn create_standalone_iso(output: &str) -> Result<()> {
    use crate::utils::*;
    use std::fs;
    
    print_header("Creating Standalone HecateOS ISO");
    
    let rust_dir = find_project_root()?;
    
    // Build all components first
    println!("\n🔨 Building all components...");
    super::build::build_all(true, false).await?;
    
    // Create ISO content directory
    let iso_content = rust_dir.join("hecate-standalone");
    if iso_content.exists() {
        fs::remove_dir_all(&iso_content)?;
    }
    
    println!("\n📦 Creating ISO structure...");
    fs::create_dir_all(iso_content.join("bin"))?;
    fs::create_dir_all(iso_content.join("usr/bin"))?;
    fs::create_dir_all(iso_content.join("etc"))?;
    fs::create_dir_all(iso_content.join("boot"))?;
    
    // Copy binaries
    println!("📥 Copying HecateOS binaries...");
    let release_dir = rust_dir.join("target/release");
    
    // Copy main binaries
    for entry in fs::read_dir(&release_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_str().unwrap_or_default();
        
        if (name.starts_with("hecate") && !name.contains('.')) {
            let dest = if name == "hecate" || name == "hecated" {
                iso_content.join("bin").join(name)
            } else {
                iso_content.join("usr/bin").join(name)
            };
            fs::copy(&path, dest)?;
        }
    }
    
    // Create info files
    fs::write(
        iso_content.join("boot/README.txt"),
        format!("HecateOS Standalone ISO\nCreated: {}\nVersion: 0.1.0", chrono::Utc::now())
    )?;
    
    fs::write(
        iso_content.join("etc/hecate-release"),
        "HECATE_VERSION=\"0.1.0\"\nHECATE_ARCH=\"x86_64\"\n"
    )?;
    
    // Build ISO using our native implementation
    println!("\n🔥 Creating ISO with native Rust...");
    let iso_builder = rust_dir.join("target/release/hecate-iso");
    
    if !iso_builder.exists() {
        build_iso_builder(&rust_dir)?;
    }
    
    let status = Command::new(&iso_builder)
        .arg("repack")
        .arg(&iso_content)
        .arg("-o").arg(output)
        .arg("--label").arg("HECATEOS")
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("ISO creation failed"));
    }
    
    // Clean up
    fs::remove_dir_all(&iso_content)?;
    
    println!("\n✅ Standalone ISO created successfully: {}", output.green());
    println!("\n📊 This ISO contains all HecateOS binaries (no Ubuntu base)");
    println!("   Size: ~40MB");
    println!("   Type: Data ISO (not bootable yet)");
    
    Ok(())
}

pub async fn create_iso(
    download: Option<&str>,
    input: Option<&str>, 
    output: &str,
    skip_build: bool,
    with_source: bool,
) -> Result<()> {
    use crate::utils::*;
    
    print_header("Creating HecateOS ISO");
    
    // If downloading and no 7z, show requirement
    if download.is_some() && !has_iso_extraction_tool() {
        eprintln!("\n╔════════════════════════════════════════════════════════════╗");
        eprintln!("║         MANDATORY REQUIREMENT: 7z Installation            ║");
        eprintln!("╚════════════════════════════════════════════════════════════╝");
        eprintln!("");
        eprintln!("HecateOS is a COMPLETE LINUX DISTRIBUTION based on Ubuntu LTS.");
        eprintln!("To build HecateOS, we MUST extract and modify Ubuntu ISO.");
        eprintln!("");
        eprintln!("⚠️  7z is MANDATORY - No alternatives, no workarounds.");
        eprintln!("");
        eprintln!("Install it now:");
        eprintln!("    {}", "sudo apt-get install p7zip-full".bright_yellow().bold());
        eprintln!("");
        eprintln!("Then run:");
        eprintln!("    {}", "hecate-dev iso create --download 24.04".bright_green());
        eprintln!("");
        eprintln!("📝 Remember: HecateOS = Ubuntu + Optimizations");
        eprintln!("   Without Ubuntu base, there is no HecateOS!");
        eprintln!("");
        return Err(anyhow::anyhow!("7z is MANDATORY. Install: sudo apt-get install p7zip-full"));
    }
    
    let rust_dir = find_project_root()?;
    let iso_builder = rust_dir.join("target/release/hecate-iso");
    
    // Build hecate-iso-builder if needed
    if !iso_builder.exists() {
        println!("Building ISO builder tool...");
        build_iso_builder(&rust_dir)?;
    }
    
    // Build all components first unless skipped
    if !skip_build {
        println!("\n🔨 Building all components...");
        super::build::build_all(true, false).await?;
    }
    
    // Prepare ISO builder command
    let mut cmd = Command::new(&iso_builder);
    cmd.current_dir(&rust_dir)
        .arg("build")
        .arg("--output").arg(output)
        .arg("--with-binaries")
        .arg("--skip-build");  // Already built by hecate-dev
    
    if let Some(version) = download {
        cmd.arg("--download").arg(version);
        cmd.arg("--input").arg("dummy"); // Required but ignored when downloading
    } else if let Some(iso) = input {
        cmd.arg("--input").arg(iso);
    } else {
        // Default to downloading latest Ubuntu
        cmd.arg("--download").arg("24.04");
        cmd.arg("--input").arg("dummy");
    }
    
    if with_source {
        cmd.arg("--with-source");
    }
    
    println!("\n📀 Creating ISO...");
    let status = cmd.status()
        .context("Failed to run ISO builder")?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("ISO creation failed"));
    }
    
    println!("\n✅ ISO created successfully: {}", output.green());
    
    // Show next steps
    println!("\n{}", "Next steps:".bright_cyan());
    println!("  1. Test in VM: qemu-system-x86_64 -m 4G -cdrom {}", output);
    println!("  2. Write to USB: sudo dd if={} of=/dev/sdX bs=4M", output);
    
    Ok(())
}

pub async fn extract_iso(iso: &str, output: &str) -> Result<()> {
    let rust_dir = find_project_root()?;
    let iso_builder = rust_dir.join("target/release/hecate-iso");
    
    if !iso_builder.exists() {
        build_iso_builder(&rust_dir)?;
    }
    
    let status = Command::new(&iso_builder)
        .current_dir(&rust_dir)
        .args(&["extract", iso, "--output", output])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("ISO extraction failed"));
    }
    
    println!("✅ ISO extracted to: {}", output);
    Ok(())
}

pub async fn verify_iso(iso: &str) -> Result<()> {
    let rust_dir = find_project_root()?;
    let iso_builder = rust_dir.join("target/release/hecate-iso");
    
    if !iso_builder.exists() {
        build_iso_builder(&rust_dir)?;
    }
    
    let status = Command::new(&iso_builder)
        .current_dir(&rust_dir)
        .args(&["verify", iso])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("ISO verification failed"));
    }
    
    Ok(())
}

pub async fn clean_artifacts() -> Result<()> {
    println!("Cleaning ISO artifacts...");
    
    // Remove common ISO artifacts
    let patterns = vec![
        "*.iso",
        "ubuntu-*.iso",
        "hecateos*.iso",
        "test-*.iso",
        "iso-extract/",
    ];
    
    for pattern in patterns {
        println!("  Removing {}", pattern);
        // Use shell expansion for globs
        Command::new("sh")
            .arg("-c")
            .arg(format!("rm -rf {}", pattern))
            .status()?;
    }
    
    println!("✅ ISO artifacts cleaned");
    Ok(())
}

fn build_iso_builder(rust_dir: &PathBuf) -> Result<()> {
    let iso_builder_dir = rust_dir.join("hecate-iso-builder");
    
    let output = Command::new("cargo")
        .current_dir(&iso_builder_dir)
        .args(&["build", "--release"])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to build ISO builder:\n{}", stderr));
    }
    
    Ok(())
}

fn find_project_root() -> Result<PathBuf> {
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
            
            // Check if installed in ~/.local/bin and find project
            if parent.ends_with(".local/bin") {
                // Look for common project locations
                let home = std::env::var("HOME").unwrap_or_default();
                let possible_paths = vec![
                    PathBuf::from(&home).join("Projects/personal/hecate-os/rust"),
                    PathBuf::from(&home).join("projects/hecate-os/rust"),
                    PathBuf::from(&home).join("hecate-os/rust"),
                    PathBuf::from("/home/akkarin/Projects/personal/hecate-os/rust"),
                ];
                
                for path in possible_paths {
                    if path.join("Makefile").exists() && path.join("hecate-daemon").is_dir() {
                        return Ok(path);
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