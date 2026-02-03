//! Integration tests for hecate-update

use hecate_update::{
    UpdateManager, UpdateConfig, UpdateType, UpdateInfo,
    SecuritySeverity, UpdateChecksum, UpdateStatus,
};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_update_manager_creation() {
    let temp_dir = tempdir().unwrap();
    let config = UpdateConfig {
        cache_dir: temp_dir.path().join("cache"),
        backup_dir: temp_dir.path().join("backups"),
        ..Default::default()
    };
    
    let manager = UpdateManager::new(config).await;
    assert!(manager.is_ok(), "Failed to create update manager");
}

#[test]
fn test_update_type_variants() {
    let kernel_update = UpdateType::KernelPatch {
        version: "6.8.0".to_string(),
        patch_level: "1".to_string(),
        requires_reboot: false,
    };
    
    assert!(matches!(kernel_update, UpdateType::KernelPatch { .. }));
    
    let driver_update = UpdateType::Driver {
        name: "nvidia".to_string(),
        version: "550.54".to_string(),
        vendor: "NVIDIA".to_string(),
        hot_swappable: true,
    };
    
    assert!(matches!(driver_update, UpdateType::Driver { .. }));
}

#[test]
fn test_security_severity_ordering() {
    assert!(SecuritySeverity::Critical > SecuritySeverity::High);
    assert!(SecuritySeverity::High > SecuritySeverity::Medium);
    assert!(SecuritySeverity::Medium > SecuritySeverity::Low);
}

#[test]
fn test_update_status() {
    let status = UpdateStatus::Available;
    assert_eq!(status, UpdateStatus::Available);
    
    let downloading = UpdateStatus::Downloading { progress: 0.5 };
    if let UpdateStatus::Downloading { progress } = downloading {
        assert_eq!(progress, 0.5);
    } else {
        panic!("Wrong status variant");
    }
}

#[tokio::test]
async fn test_update_plan_creation() {
    let temp_dir = tempdir().unwrap();
    let config = UpdateConfig {
        cache_dir: temp_dir.path().join("cache"),
        backup_dir: temp_dir.path().join("backups"),
        ..Default::default()
    };
    
    let manager = UpdateManager::new(config).await.unwrap();
    
    // With no available updates, plan should be empty
    let plan = manager.create_plan(vec![]).await;
    assert!(plan.is_ok());
    
    let plan = plan.unwrap();
    assert_eq!(plan.updates.len(), 0);
    assert_eq!(plan.requires_reboot, false);
}

#[test]
fn test_maintenance_window() {
    use hecate_update::MaintenanceWindow;
    use chrono::Weekday;
    
    let window = MaintenanceWindow {
        days: vec![Weekday::Sun, Weekday::Wed],
        start_hour: 2,
        end_hour: 6,
        timezone: "UTC".to_string(),
    };
    
    assert_eq!(window.days.len(), 2);
    assert_eq!(window.start_hour, 2);
    assert_eq!(window.end_hour, 6);
}