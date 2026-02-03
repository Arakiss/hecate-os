//! Filesystem snapshot management
//!
//! Supports BTRFS, LVM, and ZFS snapshots

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub enum SnapshotType {
    Btrfs,
    Lvm,
    Zfs,
    FileBased,
}

pub struct SnapshotManager {
    snapshot_type: SnapshotType,
    root_path: PathBuf,
}

impl SnapshotManager {
    pub fn new() -> Result<Self> {
        let snapshot_type = Self::detect_filesystem_type()?;
        
        Ok(Self {
            snapshot_type,
            root_path: PathBuf::from("/"),
        })
    }

    fn detect_filesystem_type() -> Result<SnapshotType> {
        // Check filesystem type of root
        let output = Command::new("findmnt")
            .args(&["-n", "-o", "FSTYPE", "/"])
            .output()?;
        
        let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        match fstype.as_str() {
            "btrfs" => Ok(SnapshotType::Btrfs),
            "zfs" => Ok(SnapshotType::Zfs),
            _ => {
                // Check if LVM is available
                if Path::new("/usr/sbin/lvm").exists() {
                    Ok(SnapshotType::Lvm)
                } else {
                    Ok(SnapshotType::FileBased)
                }
            }
        }
    }

    pub async fn create_snapshot(&self, name: &str) -> Result<String> {
        match self.snapshot_type {
            SnapshotType::Btrfs => self.create_btrfs_snapshot(name).await,
            SnapshotType::Lvm => self.create_lvm_snapshot(name).await,
            SnapshotType::Zfs => self.create_zfs_snapshot(name).await,
            SnapshotType::FileBased => self.create_file_snapshot(name).await,
        }
    }

    async fn create_btrfs_snapshot(&self, name: &str) -> Result<String> {
        let snapshot_path = format!("/.snapshots/{}", name);
        
        // Create snapshots directory if it doesn't exist
        std::fs::create_dir_all("/.snapshots")?;
        
        // Create BTRFS snapshot
        let output = Command::new("btrfs")
            .args(&["subvolume", "snapshot", "-r", "/", &snapshot_path])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create BTRFS snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(snapshot_path)
    }

    async fn create_lvm_snapshot(&self, name: &str) -> Result<String> {
        // Get root volume group and logical volume
        let output = Command::new("findmnt")
            .args(&["-n", "-o", "SOURCE", "/"])
            .output()?;
        
        let source = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Parse VG/LV from device path
        // Example: /dev/mapper/vg0-root -> vg0/root
        if !source.starts_with("/dev/mapper/") {
            return Err(anyhow::anyhow!("Root is not on LVM"));
        }
        
        let lv_name = source.strip_prefix("/dev/mapper/")
            .unwrap_or("")
            .replace('-', "/");
        
        // Create LVM snapshot
        let snapshot_name = format!("{}-{}", lv_name.replace('/', "-"), name);
        let output = Command::new("lvcreate")
            .args(&["-s", "-n", &snapshot_name, "-L", "5G", &lv_name])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create LVM snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(format!("/dev/mapper/{}", snapshot_name))
    }

    async fn create_zfs_snapshot(&self, name: &str) -> Result<String> {
        // Get root dataset
        let output = Command::new("zfs")
            .args(&["list", "-H", "-o", "name", "/"])
            .output()?;
        
        let dataset = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let snapshot_name = format!("{}@{}", dataset, name);
        
        // Create ZFS snapshot
        let output = Command::new("zfs")
            .args(&["snapshot", &snapshot_name])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create ZFS snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(snapshot_name)
    }

    async fn create_file_snapshot(&self, name: &str) -> Result<String> {
        // Fall back to file-based backup
        let backup_dir = format!("/var/backups/hecate-snapshots/{}", name);
        std::fs::create_dir_all(&backup_dir)?;
        
        // Save critical system files
        let critical_paths = vec![
            "/etc",
            "/boot/grub",
            "/boot/loader",
            "/usr/local",
        ];
        
        for path in critical_paths {
            if Path::new(path).exists() {
                let dest = format!("{}{}", backup_dir, path);
                if let Some(parent) = Path::new(&dest).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                // Use rsync for efficient copying
                Command::new("rsync")
                    .args(&["-av", "--quiet", path, &dest])
                    .output()?;
            }
        }
        
        // Save package list
        Command::new("dpkg")
            .args(&["-l"])
            .output()
            .ok()
            .and_then(|o| std::fs::write(format!("{}/packages.list", backup_dir), o.stdout).ok());
        
        Ok(backup_dir)
    }

    pub async fn restore_snapshot(&self, snapshot_path: &str) -> Result<()> {
        match self.snapshot_type {
            SnapshotType::Btrfs => self.restore_btrfs_snapshot(snapshot_path).await,
            SnapshotType::Lvm => self.restore_lvm_snapshot(snapshot_path).await,
            SnapshotType::Zfs => self.restore_zfs_snapshot(snapshot_path).await,
            SnapshotType::FileBased => self.restore_file_snapshot(snapshot_path).await,
        }
    }

    async fn restore_btrfs_snapshot(&self, snapshot_path: &str) -> Result<()> {
        // BTRFS snapshot restore requires reboot with different root
        // This would typically be handled by bootloader configuration
        tracing::info!("BTRFS snapshot restore requires reboot with snapshot as root");
        Ok(())
    }

    async fn restore_lvm_snapshot(&self, snapshot_path: &str) -> Result<()> {
        // LVM snapshot merge
        let output = Command::new("lvconvert")
            .args(&["--merge", snapshot_path])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to merge LVM snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        tracing::info!("LVM snapshot merge initiated, will complete on next reboot");
        Ok(())
    }

    async fn restore_zfs_snapshot(&self, snapshot_path: &str) -> Result<()> {
        // ZFS rollback
        let output = Command::new("zfs")
            .args(&["rollback", "-r", snapshot_path])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to rollback ZFS snapshot: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(())
    }

    async fn restore_file_snapshot(&self, backup_dir: &str) -> Result<()> {
        // Restore from file-based backup
        let critical_paths = vec![
            "/etc",
            "/boot/grub",
            "/boot/loader",
            "/usr/local",
        ];
        
        for path in critical_paths {
            let source = format!("{}{}", backup_dir, path);
            if Path::new(&source).exists() {
                Command::new("rsync")
                    .args(&["-av", "--delete", &source, path])
                    .output()?;
            }
        }
        
        Ok(())
    }

    pub async fn delete_snapshot(&self, snapshot_path: &str) -> Result<()> {
        match self.snapshot_type {
            SnapshotType::Btrfs => {
                Command::new("btrfs")
                    .args(&["subvolume", "delete", snapshot_path])
                    .output()?;
            }
            SnapshotType::Lvm => {
                Command::new("lvremove")
                    .args(&["-f", snapshot_path])
                    .output()?;
            }
            SnapshotType::Zfs => {
                Command::new("zfs")
                    .args(&["destroy", snapshot_path])
                    .output()?;
            }
            SnapshotType::FileBased => {
                std::fs::remove_dir_all(snapshot_path)?;
            }
        }
        
        Ok(())
    }
}