//! Component injection module

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::HecateConfig;

pub struct ComponentInjector {
    config: HecateConfig,
}

impl ComponentInjector {
    pub fn new(config: HecateConfig) -> Self {
        Self { config }
    }
    
    /// Inject compiled binaries into ISO
    pub fn inject_binaries(&self, iso_dir: &Path) -> Result<()> {
        let hecate_dir = iso_dir.join("hecateos");
        fs::create_dir_all(&hecate_dir)?;
        
        let bin_dir = hecate_dir.join("bin");
        fs::create_dir_all(&bin_dir)?;
        
        // Find Rust target directory
        let rust_dir = self.find_rust_dir()?;
        let release_dir = rust_dir.join("target/release");
        
        if !release_dir.exists() {
            eprintln!("Warning: Release binaries not found. Run 'cargo build --release' first.");
            return Ok(());
        }
        
        // Copy specified binaries
        for binary_name in &self.config.components.include_binaries {
            let src = release_dir.join(binary_name);
            if src.exists() {
                let dst = bin_dir.join(binary_name);
                fs::copy(&src, &dst)
                    .context(format!("Failed to copy {}", binary_name))?;
                
                // Make executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&dst)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dst, perms)?;
                }
            } else {
                eprintln!("Warning: Binary not found: {}", binary_name);
            }
        }
        
        Ok(())
    }
    
    /// Inject source code into ISO
    pub fn inject_source(&self, iso_dir: &Path) -> Result<()> {
        if !self.config.components.include_source {
            return Ok(());
        }
        
        let hecate_dir = iso_dir.join("hecateos");
        let source_dir = hecate_dir.join("source");
        fs::create_dir_all(&source_dir)?;
        
        let rust_dir = self.find_rust_dir()?;
        
        // Copy Rust source, excluding target directory
        for entry in WalkDir::new(&rust_dir)
            .into_iter()
            .filter_entry(|e| !e.path().to_string_lossy().contains("target"))
        {
            let entry = entry?;
            let src = entry.path();
            let relative = src.strip_prefix(&rust_dir)?;
            let dst = source_dir.join(relative);
            
            if src.is_dir() {
                fs::create_dir_all(&dst)?;
            } else {
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(src, dst)?;
            }
        }
        
        Ok(())
    }
    
    /// Inject configuration files
    pub fn inject_config(&self, iso_dir: &Path) -> Result<()> {
        let hecate_dir = iso_dir.join("hecateos");
        let config_dir = hecate_dir.join("config");
        fs::create_dir_all(&config_dir)?;
        
        // Save configuration
        let config_file = config_dir.join("hecate.toml");
        self.config.to_file(&config_file)?;
        
        // Create sysctl configuration
        let sysctl_content = self.generate_sysctl_config();
        fs::write(config_dir.join("99-hecateos.conf"), sysctl_content)?;
        
        // Create GRUB configuration snippet
        let grub_content = self.generate_grub_config();
        fs::write(config_dir.join("hecateos.cfg"), grub_content)?;
        
        // Create systemd service files
        self.create_service_files(&config_dir)?;
        
        Ok(())
    }
    
    /// Inject installer scripts
    pub fn inject_scripts(&self, iso_dir: &Path) -> Result<()> {
        let hecate_dir = iso_dir.join("hecateos");
        fs::create_dir_all(&hecate_dir)?;
        
        // Main installer script
        let installer = self.generate_installer_script();
        let installer_path = hecate_dir.join("install.sh");
        fs::write(&installer_path, installer)?;
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&installer_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&installer_path, perms)?;
        }
        
        // Additional scripts
        if let Some(script) = &self.config.scripts.post_install {
            fs::write(hecate_dir.join("post-install.sh"), script)?;
        }
        
        if let Some(script) = &self.config.scripts.first_boot {
            fs::write(hecate_dir.join("first-boot.sh"), script)?;
        }
        
        Ok(())
    }
    
    /// Modify boot configuration
    pub fn modify_boot_config(&self, iso_dir: &Path) -> Result<()> {
        // Modify GRUB configuration if it exists
        let grub_cfg = iso_dir.join("boot/grub/grub.cfg");
        if grub_cfg.exists() {
            let content = fs::read_to_string(&grub_cfg)?;
            let modified = self.modify_grub_content(&content);
            fs::write(&grub_cfg, modified)?;
        }
        
        // Modify isolinux if it exists
        let isolinux_cfg = iso_dir.join("isolinux/txt.cfg");
        if isolinux_cfg.exists() {
            let content = fs::read_to_string(&isolinux_cfg)?;
            let modified = self.modify_isolinux_content(&content);
            fs::write(&isolinux_cfg, modified)?;
        }
        
        // Update disk info
        let disk_info = iso_dir.join(".disk/info");
        if disk_info.exists() {
            let info = format!(
                "{} {} - Built on {}",
                self.config.metadata.name,
                self.config.metadata.version,
                chrono::Local::now().format("%Y-%m-%d")
            );
            fs::write(&disk_info, info)?;
        }
        
        Ok(())
    }
    
    fn find_rust_dir(&self) -> Result<PathBuf> {
        // Try to find the Rust directory
        let possible_paths = [
            PathBuf::from("../rust"),
            PathBuf::from("../../rust"),
            PathBuf::from("./rust"),
            dirs::home_dir().unwrap_or_default().join("hecateos/rust"),
        ];
        
        for path in possible_paths.iter() {
            if path.join("Cargo.toml").exists() {
                return Ok(path.canonicalize()?);
            }
        }
        
        Err(anyhow::anyhow!("Could not find Rust project directory"))
    }
    
    fn generate_sysctl_config(&self) -> String {
        let mut content = String::from("# HecateOS Performance Optimizations\n");
        
        for (key, value) in &self.config.optimizations.sysctl_settings {
            content.push_str(&format!("{} = {}\n", key, value));
        }
        
        content
    }
    
    fn generate_grub_config(&self) -> String {
        format!(
            "# HecateOS GRUB Configuration\n\
             GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"\n\
             GRUB_DISTRIBUTOR=\"{}\"\n",
            self.config.optimizations.kernel_params.join(" "),
            self.config.metadata.name
        )
    }
    
    fn generate_installer_script(&self) -> String {
        format!(
            r#"#!/bin/bash
#
# HecateOS Installer Script
# Auto-generated by hecate-iso-builder
#

set -e

echo "==========================================="
echo "  {} Installer"
echo "  Version: {}"
echo "==========================================="

# Install binaries
echo "Installing HecateOS components..."
if [ -d /cdrom/hecateos/bin ]; then
    sudo cp -v /cdrom/hecateos/bin/* /usr/local/bin/
    sudo chmod +x /usr/local/bin/hecate*
fi

# Apply system configuration
echo "Applying system optimizations..."
if [ -f /cdrom/hecateos/config/99-hecateos.conf ]; then
    sudo cp /cdrom/hecateos/config/99-hecateos.conf /etc/sysctl.d/
    sudo sysctl -p /etc/sysctl.d/99-hecateos.conf
fi

# Install systemd services
echo "Installing services..."
if [ -d /cdrom/hecateos/config/systemd ]; then
    sudo cp /cdrom/hecateos/config/systemd/*.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable hecated.service
    sudo systemctl enable hecate-monitor.service
fi

# Set CPU governor
echo "Setting CPU governor to {}..."
echo {} | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Update MOTD
sudo tee /etc/motd << 'EOF'
{}
EOF

echo ""
echo "Installation complete!"
echo "Please reboot to complete setup."
"#,
            self.config.metadata.name,
            self.config.metadata.version,
            self.config.optimizations.cpu_governor,
            self.config.optimizations.cpu_governor,
            self.config.branding.motd
        )
    }
    
    fn create_service_files(&self, config_dir: &Path) -> Result<()> {
        let systemd_dir = config_dir.join("systemd");
        fs::create_dir_all(&systemd_dir)?;
        
        // hecated.service
        let hecated_service = r#"[Unit]
Description=HecateOS System Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/hecated
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target"#;
        
        fs::write(systemd_dir.join("hecated.service"), hecated_service)?;
        
        // hecate-monitor.service
        let monitor_service = r#"[Unit]
Description=HecateOS Monitoring Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/hecate-monitor
Restart=always
RestartSec=10
Environment="HECATE_MONITOR_PORT=9313"

[Install]
WantedBy=multi-user.target"#;
        
        fs::write(systemd_dir.join("hecate-monitor.service"), monitor_service)?;
        
        Ok(())
    }
    
    fn modify_grub_content(&self, content: &str) -> String {
        // Add HecateOS branding to GRUB menu
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        
        // Find and modify menu entries
        for line in &mut lines {
            if line.contains("menuentry") && line.contains("Ubuntu") {
                *line = line.replace("Ubuntu", &self.config.metadata.name);
            }
        }
        
        lines.join("\n")
    }
    
    fn modify_isolinux_content(&self, content: &str) -> String {
        // Add HecateOS branding to isolinux menu
        content.replace("Ubuntu", &self.config.metadata.name)
    }
}