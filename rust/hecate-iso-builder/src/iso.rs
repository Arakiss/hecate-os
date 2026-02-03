//! ISO manipulation module

use anyhow::{Context, Result};
use indicatif::ProgressBar;
use std::path::Path;
use std::process::Command;
use tokio::fs;

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
        
        progress.set_message("Mounting ISO...");
        
        // Create mount point
        let mount_dir = output_dir.parent()
            .unwrap_or(Path::new("/tmp"))
            .join("iso_mount");
        fs::create_dir_all(&mount_dir).await?;
        
        // Mount ISO (requires sudo in real usage)
        let mount_output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "7z x -o{} {} 2>/dev/null || bsdtar -xf {} -C {} 2>/dev/null || (mkdir -p {} && cp -r {}/* {} 2>/dev/null)",
                output_dir.display(),
                iso_path.display(),
                iso_path.display(),
                output_dir.display(),
                mount_dir.display(),
                iso_path.display(),
                output_dir.display()
            ))
            .output()
            .context("Failed to extract ISO")?;
        
        if !mount_output.status.success() {
            // Try alternative extraction method
            progress.set_message("Trying alternative extraction...");
            
            // Use xorriso if available
            let xorriso = Command::new("xorriso")
                .arg("-osirrox")
                .arg("on")
                .arg("-indev")
                .arg(iso_path)
                .arg("-extract")
                .arg("/")
                .arg(output_dir)
                .output();
            
            if xorriso.is_err() {
                // Last resort: inform user about manual extraction
                eprintln!("\nAutomatic extraction failed. Manual steps:");
                eprintln!("  sudo mkdir -p {}", mount_dir.display());
                eprintln!("  sudo mount -o loop {} {}", iso_path.display(), mount_dir.display());
                eprintln!("  cp -r {}/* {}", mount_dir.display(), output_dir.display());
                eprintln!("  sudo umount {}", mount_dir.display());
                
                return Err(anyhow::anyhow!("ISO extraction failed. See manual steps above."));
            }
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
        progress.set_message("Creating ISO structure...");
        
        // Check for required tools
        let tools = ["xorriso", "genisoimage", "mkisofs"];
        let mut iso_tool = None;
        
        for tool in &tools {
            if Command::new("which").arg(tool).output()?.status.success() {
                iso_tool = Some(tool.to_string());
                break;
            }
        }
        
        let tool = iso_tool.ok_or_else(|| {
            anyhow::anyhow!("No ISO creation tool found. Install xorriso or genisoimage")
        })?;
        
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