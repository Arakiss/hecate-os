//! HecateOS Intelligent Update System
//! 
//! Provides live kernel patching, driver hot-swapping, automatic rollback,
//! and workload-aware scheduling for system updates.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use semver::Version;
use chrono::{DateTime, Utc, Local};
use async_trait::async_trait;

pub mod kernel;
pub mod driver;
pub mod rollback;
pub mod scheduler;
pub mod snapshot;

// ============================================================================
// UPDATE TYPES AND METADATA
// ============================================================================

/// Type of system update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateType {
    KernelPatch {
        version: String,
        patch_level: String,
        requires_reboot: bool,
    },
    Driver {
        name: String,
        version: String,
        vendor: String,
        hot_swappable: bool,
    },
    Package {
        name: String,
        version: Version,
    },
    Firmware {
        component: String,
        version: String,
        requires_reboot: bool,
    },
    Security {
        cve_id: String,
        severity: SecuritySeverity,
        immediate: bool,
    },
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Update metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub id: String,
    pub update_type: UpdateType,
    pub description: String,
    pub size_bytes: u64,
    pub download_url: String,
    pub checksum: UpdateChecksum,
    pub signature: Option<String>,
    pub release_date: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub changelog: Option<String>,
}

/// Update checksum for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChecksum {
    pub sha256: String,
    pub blake3: String,
}

/// Update status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateStatus {
    Available,
    Downloading { progress: f32 },
    Downloaded,
    Verifying,
    Preparing,
    Installing { progress: f32 },
    Installed,
    Failed { error: String },
    RolledBack,
}

/// System update plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlan {
    pub updates: Vec<UpdateInfo>,
    pub order: Vec<String>,  // Update IDs in installation order
    pub estimated_time: std::time::Duration,
    pub requires_reboot: bool,
    pub snapshot_before: bool,
    pub auto_rollback: bool,
}

// ============================================================================
// UPDATE MANAGER
// ============================================================================

/// Main update manager
pub struct UpdateManager {
    config: UpdateConfig,
    kernel_manager: kernel::KernelPatchManager,
    driver_manager: driver::DriverManager,
    rollback_manager: rollback::RollbackManager,
    scheduler: scheduler::UpdateScheduler,
    state: UpdateState,
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub update_server: String,
    pub cache_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub enable_live_patching: bool,
    pub enable_hot_swapping: bool,
    pub auto_rollback: bool,
    pub rollback_timeout: std::time::Duration,
    pub schedule_updates: bool,
    pub maintenance_window: MaintenanceWindow,
    pub max_parallel_downloads: usize,
    pub verify_signatures: bool,
}

/// Maintenance window for scheduled updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub days: Vec<chrono::Weekday>,
    pub start_hour: u32,
    pub end_hour: u32,
    pub timezone: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            update_server: "https://updates.hecateos.org".to_string(),
            cache_dir: PathBuf::from("/var/cache/hecate-update"),
            backup_dir: PathBuf::from("/var/backups/hecate-update"),
            enable_live_patching: true,
            enable_hot_swapping: true,
            auto_rollback: true,
            rollback_timeout: std::time::Duration::from_secs(300),
            schedule_updates: false,
            maintenance_window: MaintenanceWindow {
                days: vec![chrono::Weekday::Sun, chrono::Weekday::Wed],
                start_hour: 2,
                end_hour: 6,
                timezone: "UTC".to_string(),
            },
            max_parallel_downloads: 4,
            verify_signatures: true,
        }
    }
}

/// Internal update state
struct UpdateState {
    available_updates: HashMap<String, UpdateInfo>,
    installed_updates: HashSet<String>,
    pending_updates: Vec<String>,
    active_snapshot: Option<String>,
}

impl UpdateManager {
    /// Create new update manager
    pub async fn new(config: UpdateConfig) -> Result<Self> {
        // Ensure directories exist
        std::fs::create_dir_all(&config.cache_dir)?;
        std::fs::create_dir_all(&config.backup_dir)?;

        let kernel_manager = kernel::KernelPatchManager::new()?;
        let driver_manager = driver::DriverManager::new()?;
        let rollback_manager = rollback::RollbackManager::new(&config.backup_dir)?;
        let scheduler = scheduler::UpdateScheduler::new(config.maintenance_window.clone())?;

        let state = UpdateState {
            available_updates: HashMap::new(),
            installed_updates: HashSet::new(),
            pending_updates: Vec::new(),
            active_snapshot: None,
        };

        Ok(Self {
            config,
            kernel_manager,
            driver_manager,
            rollback_manager,
            scheduler,
            state,
        })
    }

    /// Check for available updates
    pub async fn check_updates(&mut self) -> Result<Vec<UpdateInfo>> {
        tracing::info!("Checking for system updates...");

        let mut all_updates = Vec::new();

        // Check kernel updates
        let kernel_updates = self.check_kernel_updates().await?;
        all_updates.extend(kernel_updates);

        // Check driver updates
        let driver_updates = self.check_driver_updates().await?;
        all_updates.extend(driver_updates);

        // Check package updates
        let package_updates = self.check_package_updates().await?;
        all_updates.extend(package_updates);

        // Check firmware updates
        let firmware_updates = self.check_firmware_updates().await?;
        all_updates.extend(firmware_updates);

        // Store in state
        for update in &all_updates {
            self.state.available_updates.insert(update.id.clone(), update.clone());
        }

        tracing::info!("Found {} available updates", all_updates.len());
        Ok(all_updates)
    }

    /// Create an update plan
    pub async fn create_plan(&self, update_ids: Vec<String>) -> Result<UpdatePlan> {
        let mut updates = Vec::new();
        let mut requires_reboot = false;

        for id in &update_ids {
            let update = self.state.available_updates.get(id)
                .ok_or_else(|| anyhow::anyhow!("Update {} not found", id))?;
            
            updates.push(update.clone());

            // Check if reboot required
            match &update.update_type {
                UpdateType::KernelPatch { requires_reboot: r, .. } |
                UpdateType::Firmware { requires_reboot: r, .. } => {
                    if *r {
                        requires_reboot = true;
                    }
                }
                _ => {}
            }
        }

        // Determine installation order based on dependencies
        let order = self.resolve_dependencies(&updates)?;

        // Estimate time
        let estimated_time = self.estimate_update_time(&updates);

        Ok(UpdatePlan {
            updates,
            order,
            estimated_time,
            requires_reboot,
            snapshot_before: self.config.auto_rollback,
            auto_rollback: self.config.auto_rollback,
        })
    }

    /// Apply updates according to plan
    pub async fn apply_updates(&mut self, plan: UpdatePlan) -> Result<()> {
        tracing::info!("Starting update process with {} updates", plan.updates.len());

        // Create snapshot if requested
        if plan.snapshot_before {
            let snapshot_id = self.create_snapshot().await?;
            self.state.active_snapshot = Some(snapshot_id);
        }

        // Apply updates in order
        for update_id in &plan.order {
            let update = plan.updates.iter()
                .find(|u| u.id == *update_id)
                .ok_or_else(|| anyhow::anyhow!("Update {} not in plan", update_id))?;

            match self.apply_single_update(update).await {
                Ok(()) => {
                    tracing::info!("Successfully applied update: {}", update_id);
                    self.state.installed_updates.insert(update_id.clone());
                }
                Err(e) => {
                    tracing::error!("Failed to apply update {}: {}", update_id, e);
                    
                    if plan.auto_rollback {
                        self.rollback().await?;
                    }
                    
                    return Err(e);
                }
            }
        }

        // Clear active snapshot on success
        self.state.active_snapshot = None;

        // Schedule reboot if needed
        if plan.requires_reboot {
            self.schedule_reboot().await?;
        }

        Ok(())
    }

    /// Apply a single update
    async fn apply_single_update(&mut self, update: &UpdateInfo) -> Result<()> {
        match &update.update_type {
            UpdateType::KernelPatch { version, requires_reboot, .. } => {
                if self.config.enable_live_patching && !requires_reboot {
                    self.kernel_manager.apply_live_patch(update).await?;
                } else {
                    self.kernel_manager.prepare_update(update).await?;
                    self.state.pending_updates.push(update.id.clone());
                }
            }
            UpdateType::Driver { name, hot_swappable, .. } => {
                if self.config.enable_hot_swapping && *hot_swappable {
                    self.driver_manager.hot_swap(update).await?;
                } else {
                    self.driver_manager.prepare_update(update).await?;
                    self.state.pending_updates.push(update.id.clone());
                }
            }
            UpdateType::Package { name, version } => {
                self.apply_package_update(name, version).await?;
            }
            UpdateType::Firmware { component, .. } => {
                self.apply_firmware_update(update).await?;
                self.state.pending_updates.push(update.id.clone());
            }
            UpdateType::Security { immediate, .. } => {
                if *immediate {
                    self.apply_security_update(update).await?;
                } else {
                    self.state.pending_updates.push(update.id.clone());
                }
            }
        }

        Ok(())
    }

    /// Rollback recent updates
    pub async fn rollback(&mut self) -> Result<()> {
        tracing::warn!("Initiating rollback...");

        if let Some(snapshot_id) = &self.state.active_snapshot {
            self.rollback_manager.rollback_to_snapshot(snapshot_id).await?;
            self.state.active_snapshot = None;
            tracing::info!("Rollback completed successfully");
        } else {
            return Err(anyhow::anyhow!("No active snapshot for rollback"));
        }

        Ok(())
    }

    /// Schedule updates for maintenance window
    pub async fn schedule_updates(&mut self, update_ids: Vec<String>) -> Result<()> {
        let plan = self.create_plan(update_ids).await?;
        self.scheduler.schedule_plan(plan).await?;
        Ok(())
    }

    /// Get update history
    pub async fn get_history(&self) -> Result<Vec<UpdateHistory>> {
        self.rollback_manager.get_history().await
    }

    // ========================================================================
    // PRIVATE METHODS
    // ========================================================================

    async fn check_kernel_updates(&self) -> Result<Vec<UpdateInfo>> {
        self.kernel_manager.check_updates(&self.config.update_server).await
    }

    async fn check_driver_updates(&self) -> Result<Vec<UpdateInfo>> {
        self.driver_manager.check_updates(&self.config.update_server).await
    }

    async fn check_package_updates(&self) -> Result<Vec<UpdateInfo>> {
        // Integration with hecate-pkg
        // TODO: Implement package update checking
        Ok(Vec::new())
    }

    async fn check_firmware_updates(&self) -> Result<Vec<UpdateInfo>> {
        // Check for firmware updates
        // TODO: Implement firmware update checking
        Ok(Vec::new())
    }

    async fn create_snapshot(&mut self) -> Result<String> {
        self.rollback_manager.create_snapshot().await
    }

    async fn apply_package_update(&self, name: &str, version: &Version) -> Result<()> {
        // Use hecate-pkg to update package
        // TODO: Implement package update
        Ok(())
    }

    async fn apply_firmware_update(&self, update: &UpdateInfo) -> Result<()> {
        // Apply firmware update
        // TODO: Implement firmware update
        Ok(())
    }

    async fn apply_security_update(&self, update: &UpdateInfo) -> Result<()> {
        // Apply security update immediately
        // TODO: Implement security update
        Ok(())
    }

    async fn schedule_reboot(&self) -> Result<()> {
        // Schedule system reboot
        tracing::info!("Scheduling system reboot...");
        // TODO: Implement reboot scheduling
        Ok(())
    }

    fn resolve_dependencies(&self, updates: &[UpdateInfo]) -> Result<Vec<String>> {
        // Simple topological sort for dependencies
        let mut order = Vec::new();
        let mut visited = HashSet::new();

        for update in updates {
            self.visit_update(&update.id, updates, &mut visited, &mut order)?;
        }

        Ok(order)
    }

    fn visit_update(
        &self,
        id: &str,
        updates: &[UpdateInfo],
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if visited.contains(id) {
            return Ok(());
        }

        let update = updates.iter()
            .find(|u| u.id == id)
            .ok_or_else(|| anyhow::anyhow!("Update {} not found", id))?;

        // Visit dependencies first
        for dep in &update.dependencies {
            if updates.iter().any(|u| u.id == *dep) {
                self.visit_update(dep, updates, visited, order)?;
            }
        }

        visited.insert(id.to_string());
        order.push(id.to_string());

        Ok(())
    }

    fn estimate_update_time(&self, updates: &[UpdateInfo]) -> std::time::Duration {
        let mut total_seconds = 0u64;

        for update in updates {
            // Estimate based on size and type
            let base_time = (update.size_bytes / (10 * 1024 * 1024)) as u64; // 10MB/s estimate
            
            let multiplier = match &update.update_type {
                UpdateType::KernelPatch { .. } => 2,
                UpdateType::Driver { .. } => 2,
                UpdateType::Firmware { .. } => 3,
                _ => 1,
            };

            total_seconds += base_time * multiplier + 30; // Add 30s overhead per update
        }

        std::time::Duration::from_secs(total_seconds)
    }
}

/// Update history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHistory {
    pub id: String,
    pub update_type: UpdateType,
    pub timestamp: DateTime<Utc>,
    pub status: UpdateStatus,
    pub duration: std::time::Duration,
    pub rollback_available: bool,
}
