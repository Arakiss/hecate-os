use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Install 7z binary directly without using package manager
pub async fn install_7z_standalone() -> Result<()> {
    println!("{}", "Installing standalone 7z...".bright_cyan());
    
    let temp_dir = tempfile::tempdir()?;
    let download_path = temp_dir.path().join("7z.tar.xz");
    
    // Download 7z binary
    println!("  📥 Downloading 7z...");
    let download_url = "https://www.7-zip.org/a/7z2301-linux-x64.tar.xz";
    
    let output = Command::new("wget")
        .arg("-q")
        .arg("-O")
        .arg(&download_path)
        .arg(download_url)
        .output()
        .context("Failed to download 7z. Is wget installed?")?;
    
    if !output.status.success() {
        // Try with curl
        let output = Command::new("curl")
            .arg("-L")
            .arg("-o")
            .arg(&download_path)
            .arg(download_url)
            .output()
            .context("Failed to download 7z. Install wget or curl")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to download 7z"));
        }
    }
    
    // Extract 7z
    println!("  📦 Extracting 7z...");
    let output = Command::new("tar")
        .arg("xf")
        .arg(&download_path)
        .current_dir(temp_dir.path())
        .output()
        .context("Failed to extract 7z")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to extract 7z archive"));
    }
    
    // Install to user's local bin
    let local_bin = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".local/bin");
    
    fs::create_dir_all(&local_bin)?;
    
    let source_7z = temp_dir.path().join("7zz");
    let dest_7z = local_bin.join("7z");
    
    if source_7z.exists() {
        fs::copy(&source_7z, &dest_7z)
            .context("Failed to copy 7z to ~/.local/bin")?;
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_7z)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_7z, perms)?;
        }
        
        println!("  ✅ 7z installed to: {}", dest_7z.display());
        println!("  ℹ️  Make sure ~/.local/bin is in your PATH");
        
        // Check if ~/.local/bin is in PATH
        if let Ok(path) = std::env::var("PATH") {
            if !path.contains(".local/bin") {
                println!("\n  {} Add to your shell profile:", "NOTE:".yellow());
                println!("  export PATH=\"$HOME/.local/bin:$PATH\"");
            }
        }
    } else {
        return Err(anyhow::anyhow!("7z binary not found in archive"));
    }
    
    Ok(())
}

/// Setup all required tools for HecateOS development
pub async fn setup_environment(auto_install: bool) -> Result<()> {
    use crate::utils::*;
    
    print_header("HecateOS Environment Setup");
    
    // Check what's missing
    let deps = check_dependencies()?;
    
    let mut missing_tools = Vec::new();
    
    if !deps.get("7z").map_or(false, |s| s.installed) {
        missing_tools.push("7z");
    }
    
    if !deps.get("xorriso").map_or(false, |s| s.installed) {
        missing_tools.push("xorriso");
    }
    
    if missing_tools.is_empty() {
        success_msg("All required tools are already installed!");
        return Ok(());
    }
    
    println!("\n{}", "Missing Tools:".yellow());
    for tool in &missing_tools {
        println!("  • {}", tool);
    }
    
    if !auto_install {
        println!("\n{}", "Installation Options:".bright_cyan());
        println!("1. Use package manager (requires sudo):");
        println!("   {}", "sudo apt-get install p7zip-full xorriso".bright_yellow());
        println!("\n2. Install standalone (no sudo required):");
        println!("   {}", "hecate-dev setup --auto".bright_yellow());
        return Ok(());
    }
    
    // Auto-install mode
    for tool in missing_tools {
        match tool {
            "7z" => {
                println!("\nInstalling 7z (standalone)...");
                if let Err(e) = install_7z_standalone().await {
                    error_msg(&format!("Failed to install 7z: {}", e));
                    println!("  Try manual installation: sudo apt-get install p7zip-full");
                } else {
                    success_msg("7z installed successfully");
                }
            }
            "xorriso" => {
                println!("\nxorriso requires package manager installation:");
                println!("  {}", "sudo apt-get install xorriso".bright_yellow());
            }
            _ => {}
        }
    }
    
    // Verify installation
    println!("\n{}", "Verification:".bright_cyan());
    let new_deps = check_dependencies()?;
    
    if new_deps.get("7z").map_or(false, |s| s.installed) ||
       new_deps.get("bsdtar").map_or(false, |s| s.installed) {
        success_msg("ISO extraction tools ready!");
    } else {
        warn_msg("ISO extraction tools still missing");
    }
    
    Ok(())
}