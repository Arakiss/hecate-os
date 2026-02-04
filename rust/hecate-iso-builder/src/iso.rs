//! ISO manipulation module

use anyhow::{Context, Result};
use indicatif::ProgressBar;
use std::path::Path;
use std::process::Command;
use tokio::fs;
use colored::Colorize;

pub struct IsoManager;

impl IsoManager {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract an ISO to a directory
    pub async fn extract(
        &self, 
        iso_path: &Path, 
        output_dir: &Path,
        progress: &ProgressBar,
    ) -> Result<()> {
        // Create output directory
        fs::create_dir_all(output_dir).await?;
        
        progress.set_message("Extracting ISO with native Rust implementation...");
        
        // Try native extraction first
        use crate::iso_extractor;
        
        match iso_extractor::extract_iso(iso_path, output_dir) {
            Ok(_) => {
                progress.set_message("✅ ISO extracted successfully with native Rust!");
                progress.inc(100);
                return Ok(());
            }
            Err(e) => {
                // Native extraction failed, try external tools
                progress.set_message("Native extraction failed, trying external tools...");
                eprintln!("Native extraction error: {}", e);
            }
        }
        
        // Fallback to external tools
        progress.set_message("Trying external extraction tools...");
        
        // Create mount point
        let mount_dir = output_dir.parent()
            .unwrap_or(Path::new("/tmp"))
            .join("iso_mount");
        fs::create_dir_all(&mount_dir).await?;
        
        // Try extraction methods that don't require sudo
        // First try 7z
        let result = Command::new("7z")
            .arg("x")
            .arg(format!("-o{}", output_dir.display()))
            .arg("-y")
            .arg(iso_path)
            .output();
        
        let success = if let Ok(output) = result {
            output.status.success()
        } else {
            false
        };
        
        if !success {
            // All automatic extraction methods failed
            eprintln!("\n╔════════════════════════════════════════════════════════════╗");
            eprintln!("║         ISO Extraction Tool Required                      ║");
            eprintln!("╚════════════════════════════════════════════════════════════╝");
            eprintln!("");
            eprintln!("📝 Note: While HecateOS can CREATE ISOs natively in Rust,");
            eprintln!("   extracting existing ISOs (like Ubuntu) still requires a tool.");
            eprintln!("");
            eprintln!("🔧 Quick fix: Install 7z (recommended):");
            eprintln!("   {}", "sudo apt-get install p7zip-full".bright_yellow());
            eprintln!("");
            eprintln!("📚 Why? Extracting ISOs requires reading complex filesystem");
            eprintln!("   structures. Native extraction is coming in a future update!");
            eprintln!("");
            eprintln!("Alternative manual extraction:");
            eprintln!("  sudo mkdir -p {}", mount_dir.display());
            eprintln!("  sudo mount -o loop {} {}", iso_path.display(), mount_dir.display());
            eprintln!("  cp -r {}/* {}", mount_dir.display(), output_dir.display());
            eprintln!("  sudo umount {}", mount_dir.display());
            
            return Err(anyhow::anyhow!("ISO extraction requires 7z. Install with: sudo apt-get install p7zip-full"));
        }
        
        progress.set_message("Copying hidden files...");
        
        // Copy hidden files like .disk
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("cp -r {}/.* {} 2>/dev/null || true", 
                mount_dir.display(), 
                output_dir.display()
            ))
            .output();
        
        // Cleanup mount point
        let _ = fs::remove_dir_all(&mount_dir).await;
        
        progress.inc(100);
        Ok(())
    }
    
    /// Repack a directory into an ISO
    pub async fn repack(
        &self,
        source_dir: &Path,
        output_iso: &Path,
        volume_id: &str,
        progress: &ProgressBar,
    ) -> Result<()> {
        progress.set_message("Creating ISO with native Rust implementation...");
        
        // Try native implementation first
        use crate::iso_native::NativeIsoBuilder;
        
        let mut builder = NativeIsoBuilder::new(volume_id.to_string());
        match builder.add_directory_tree(source_dir, "/") {
            Ok(_) => {
                match builder.build(output_iso) {
                    Ok(_) => {
                        progress.set_message("✅ ISO created successfully with native Rust!");
                        progress.inc(100);
                        return Ok(());
                    }
                    Err(_e) => {
                        // Native failed, try external tools
                        progress.set_message("Trying external tools as fallback...");
                    }
                }
            }
            Err(_e) => {
                // Native failed, try external tools
                progress.set_message("Trying external tools as fallback...");
            }
        }
        
        // Fallback: Check for external tools
        let tools = ["xorriso", "genisoimage", "mkisofs"];
        let mut iso_tool = None;
        
        for tool in &tools {
            if Command::new("which").arg(tool).output()?.status.success() {
                iso_tool = Some(tool.to_string());
                break;
            }
        }
        
        // If no ISO tool found, create tar.gz as fallback
        let tool = if let Some(t) = iso_tool {
            t
        } else {
            eprintln!("\n⚠️  No ISO creation tool found!");
            eprintln!("Creating a tar.gz archive instead of ISO...\n");
            
            // Create tar.gz as fallback
            let tar_name = output_iso.with_extension("tar.gz");
            progress.set_message("Creating tar.gz archive...");
            
            let output = Command::new("tar")
                .arg("czf")
                .arg(&tar_name)
                .arg("-C")
                .arg(source_dir.parent().unwrap_or(Path::new(".")))
                .arg(source_dir.file_name().unwrap_or(std::ffi::OsStr::new(".")))
                .output()?;
                
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to create tar.gz archive"));
            }
            
            progress.inc(100);
            
            eprintln!("\n✅ Created archive: {}", tar_name.display());
            eprintln!("\nTo create a proper ISO, install one of:");
            eprintln!("  • sudo apt-get install xorriso");
            eprintln!("  • sudo apt-get install genisoimage");
            
            return Ok(());
        };
        
        progress.set_message("Building ISO...");
        
        match tool.as_str() {
            "xorriso" => {
                self.repack_with_xorriso(source_dir, output_iso, volume_id).await?;
            }
            "genisoimage" | "mkisofs" => {
                self.repack_with_genisoimage(source_dir, output_iso, volume_id, &tool).await?;
            }
            _ => unreachable!(),
        }
        
        progress.inc(100);
        Ok(())
    }
    
    async fn repack_with_xorriso(
        &self,
        source_dir: &Path,
        output_iso: &Path,
        volume_id: &str,
    ) -> Result<()> {
        // Check if this is UEFI or legacy boot
        let is_uefi = source_dir.join("EFI").exists();
        let is_legacy = source_dir.join("isolinux").exists();
        
        let mut cmd = Command::new("xorriso");
        cmd.arg("-as").arg("mkisofs");
        
        // Basic options
        cmd.arg("-r")
           .arg("-J")
           .arg("-joliet-long")
           .arg("-V").arg(volume_id)
           .arg("-o").arg(output_iso);
        
        // Legacy BIOS boot
        if is_legacy {
            if let Ok(isolinux_bin) = which::which("isolinux.bin") {
                cmd.arg("-isohybrid-mbr").arg(isolinux_bin);
            }
            
            cmd.arg("-b").arg("isolinux/isolinux.bin")
               .arg("-c").arg("isolinux/boot.cat")
               .arg("-no-emul-boot")
               .arg("-boot-load-size").arg("4")
               .arg("-boot-info-table");
        }
        
        // UEFI boot
        if is_uefi {
            let efi_img = source_dir.join("boot/grub/efi.img");
            if efi_img.exists() {
                cmd.arg("-eltorito-alt-boot")
                   .arg("-e").arg("boot/grub/efi.img")
                   .arg("-no-emul-boot")
                   .arg("-isohybrid-gpt-basdat");
            }
        }
        
        cmd.arg(source_dir);
        
        let output = cmd.output().context("Failed to run xorriso")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("xorriso failed: {}", stderr));
        }
        
        Ok(())
    }
    
    async fn repack_with_genisoimage(
        &self,
        source_dir: &Path,
        output_iso: &Path,
        volume_id: &str,
        tool_name: &str,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_name);
        
        // Basic options
        cmd.arg("-r")
           .arg("-J")
           .arg("-joliet-long")
           .arg("-V").arg(volume_id)
           .arg("-o").arg(output_iso);
        
        // Check for isolinux
        let isolinux_cfg = source_dir.join("isolinux/isolinux.cfg");
        if isolinux_cfg.exists() {
            cmd.arg("-b").arg("isolinux/isolinux.bin")
               .arg("-c").arg("isolinux/boot.cat")
               .arg("-no-emul-boot")
               .arg("-boot-load-size").arg("4")
               .arg("-boot-info-table");
        }
        
        cmd.arg(source_dir);
        
        let output = cmd.output()
            .context(format!("Failed to run {}", tool_name))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("{} failed: {}", tool_name, stderr));
        }
        
        // Make hybrid if possible
        if Command::new("which").arg("isohybrid").output()?.status.success() {
            let _ = Command::new("isohybrid")
                .arg(output_iso)
                .output();
        }
        
        Ok(())
    }
}