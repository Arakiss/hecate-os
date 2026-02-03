//! Configuration module for HecateOS ISO builder

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HecateConfig {
    pub metadata: Metadata,
    pub components: Components,
    pub optimizations: Optimizations,
    pub branding: Branding,
    pub scripts: Scripts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub base_distro: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub include_binaries: Vec<String>,
    pub include_source: bool,
    pub include_docs: bool,
    pub additional_packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimizations {
    pub kernel_params: Vec<String>,
    pub sysctl_settings: Vec<(String, String)>,
    pub cpu_governor: String,
    pub io_scheduler: String,
    pub transparent_hugepages: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branding {
    pub distro_name: String,
    pub distro_version: String,
    pub boot_splash: Option<String>,
    pub wallpaper: Option<String>,
    pub motd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scripts {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub first_boot: Option<String>,
}

impl Default for HecateConfig {
    fn default() -> Self {
        Self {
            metadata: Metadata {
                name: "HecateOS".to_string(),
                version: "0.1.0".to_string(),
                base_distro: "Ubuntu 24.04 LTS".to_string(),
                architecture: "amd64".to_string(),
            },
            components: Components {
                include_binaries: vec![
                    "hecated".to_string(),
                    "hecate-monitor".to_string(),
                    "hecate-bench".to_string(),
                    "hecate-pkg".to_string(),
                    "hecate-dev".to_string(),
                    "hecate-sign".to_string(),
                ],
                include_source: false,
                include_docs: true,
                additional_packages: vec![
                    "build-essential".to_string(),
                    "cpufrequtils".to_string(),
                    "tuned".to_string(),
                    "irqbalance".to_string(),
                    "nvme-cli".to_string(),
                ],
            },
            optimizations: Optimizations {
                kernel_params: vec![
                    "mitigations=off".to_string(),
                    "intel_pstate=enable".to_string(),
                    "transparent_hugepage=always".to_string(),
                    "processor.max_cstate=1".to_string(),
                ],
                sysctl_settings: vec![
                    ("vm.swappiness".to_string(), "10".to_string()),
                    ("vm.vfs_cache_pressure".to_string(), "50".to_string()),
                    ("vm.dirty_background_ratio".to_string(), "5".to_string()),
                    ("vm.dirty_ratio".to_string(), "10".to_string()),
                    ("net.core.default_qdisc".to_string(), "fq_codel".to_string()),
                    ("net.ipv4.tcp_congestion".to_string(), "bbr".to_string()),
                    ("kernel.sched_autogroup_enabled".to_string(), "1".to_string()),
                ],
                cpu_governor: "performance".to_string(),
                io_scheduler: "bfq".to_string(),
                transparent_hugepages: "always".to_string(),
            },
            branding: Branding {
                distro_name: "HecateOS".to_string(),
                distro_version: "0.1.0 Performance Beast".to_string(),
                boot_splash: None,
                wallpaper: None,
                motd: r#"
 ██╗  ██╗███████╗ ██████╗ █████╗ ████████╗███████╗ ██████╗ ███████╗
 ██║  ██║██╔════╝██╔════╝██╔══██╗╚══██╔══╝██╔════╝██╔═══██╗██╔════╝
 ███████║█████╗  ██║     ███████║   ██║   █████╗  ██║   ██║███████╗
 ██╔══██║██╔══╝  ██║     ██╔══██║   ██║   ██╔══╝  ██║   ██║╚════██║
 ██║  ██║███████╗╚██████╗██║  ██║   ██║   ███████╗╚██████╔╝███████║
 ╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝ ╚═════╝ ╚══════╝
                                                         
 Performance-Optimized Linux Distribution
 Monitor Dashboard: http://localhost:9313
"#.to_string(),
            },
            scripts: Scripts {
                pre_install: None,
                post_install: Some(include_str!("scripts/post_install.sh").to_string()),
                first_boot: Some(include_str!("scripts/first_boot.sh").to_string()),
            },
        }
    }
}

impl HecateConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}