//! Integration tests for hecate-pkg

use hecate_pkg::{PackageManager, PackageConfig, Package, Architecture};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_package_manager_creation() {
    let temp_dir = tempdir().unwrap();
    let config = PackageConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: temp_dir.path().join("db"),
        cache_dir: temp_dir.path().join("cache"),
        log_dir: temp_dir.path().join("logs"),
        ..Default::default()
    };
    
    let manager = PackageManager::new(config).await;
    assert!(manager.is_ok(), "Failed to create package manager");
}

#[tokio::test]
async fn test_search_packages() {
    let temp_dir = tempdir().unwrap();
    let config = PackageConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: temp_dir.path().join("db"),
        cache_dir: temp_dir.path().join("cache"),
        log_dir: temp_dir.path().join("logs"),
        ..Default::default()
    };
    
    let manager = PackageManager::new(config).await.unwrap();
    let results = manager.search("test").await.unwrap();
    
    // Should return empty results for fresh database
    assert_eq!(results.len(), 0);
}

#[test]
fn test_package_metadata() {
    let package = Package {
        name: "test-package".to_string(),
        version: semver::Version::parse("1.0.0").unwrap(),
        description: "Test package".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        homepage: None,
        repository: None,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        provides: Vec::new(),
        replaces: Vec::new(),
        categories: vec!["test".to_string()],
        keywords: vec!["test".to_string()],
        architecture: Architecture::X86_64,
        size_bytes: 1024,
        installed_size_bytes: 2048,
        checksum: hecate_pkg::PackageChecksum {
            sha256: "abc123".to_string(),
            blake3: "def456".to_string(),
        },
        signature: None,
        build_date: chrono::Utc::now(),
    };
    
    assert_eq!(package.name, "test-package");
    assert_eq!(package.version.to_string(), "1.0.0");
}

#[test]
fn test_architecture_detection() {
    let arch = Architecture::X86_64;
    assert_eq!(arch, Architecture::X86_64);
    
    let arch_all = Architecture::All;
    assert_eq!(arch_all, Architecture::All);
}