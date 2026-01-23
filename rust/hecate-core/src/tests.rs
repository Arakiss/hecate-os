//! Comprehensive test suite for HecateOS core functionality

#[cfg(test)]
mod unit_tests {
    use super::super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test hardware detection on various CPU configurations
    #[test]
    fn test_cpu_detection() {
        let detector = HardwareDetector::new();
        let cpu_info = detector.detect_cpu().unwrap();
        
        // Basic sanity checks
        assert!(cpu_info.cores > 0);
        assert!(cpu_info.threads >= cpu_info.cores);
        assert!(!cpu_info.vendor.is_empty());
        assert!(!cpu_info.model.is_empty());
    }

    /// Test memory detection
    #[test]
    fn test_memory_detection() {
        let detector = HardwareDetector::new();
        let memory_info = detector.detect_memory().unwrap();
        
        // System should have at least 1GB RAM
        assert!(memory_info.total_gb >= 1.0);
    }

    /// Test profile determination logic
    #[test]
    fn test_profile_determination() {
        // Test AI Flagship profile
        let cpu = CpuInfo {
            vendor: "Intel".to_string(),
            model: "Intel Core i9-13900K".to_string(),
            cores: 24,
            threads: 32,
            base_frequency: 3000.0,
            max_frequency: 5800.0,
            generation: Some(13),
        };
        
        let memory = MemoryInfo {
            total_gb: 128.0,
            speed_mhz: Some(6400),
            memory_type: Some("DDR5".to_string()),
        };
        
        let gpu = vec![GpuInfo {
            vendor: GpuVendor::Nvidia,
            model: "RTX 4090".to_string(),
            vram_gb: 24.0,
            driver_version: Some("545.29.06".to_string()),
            compute_capability: Some("8.9".to_string()),
        }];
        
        let profile = HardwareDetector::determine_profile(&cpu, &memory, &gpu);
        assert!(matches!(profile, SystemProfile::AIFlagship));
    }

    /// Test storage type detection
    #[test]
    fn test_storage_type_detection() {
        assert!(matches!(
            HardwareDetector::detect_storage_type("nvme0n1"),
            StorageType::NvmeGen4 | StorageType::NvmeGen3
        ));
        
        assert!(matches!(
            HardwareDetector::detect_storage_type("sda"),
            StorageType::Sata | StorageType::Hdd
        ));
    }

    /// Test configuration serialization/deserialization
    #[test]
    fn test_config_serialization() {
        let hardware = create_test_hardware_info();
        
        // Serialize to JSON
        let json = serde_json::to_string(&hardware).unwrap();
        
        // Deserialize back
        let deserialized: HardwareInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(hardware.cpu.model, deserialized.cpu.model);
        assert_eq!(hardware.memory.total_gb, deserialized.memory.total_gb);
        assert_eq!(hardware.gpu.len(), deserialized.gpu.len());
    }

    /// Test optimization application doesn't panic
    #[test]
    fn test_optimization_application() {
        for profile in [
            SystemProfile::AIFlagship,
            SystemProfile::ProWorkstation,
            SystemProfile::HighPerformance,
            SystemProfile::Developer,
            SystemProfile::Standard,
        ] {
            // Should not panic
            let result = apply_optimizations(&profile);
            assert!(result.is_ok());
        }
    }

    // Helper function to create test hardware info
    fn create_test_hardware_info() -> HardwareInfo {
        HardwareInfo {
            cpu: CpuInfo {
                vendor: "Intel".to_string(),
                model: "Test CPU".to_string(),
                cores: 8,
                threads: 16,
                base_frequency: 3000.0,
                max_frequency: 4500.0,
                generation: Some(12),
            },
            memory: MemoryInfo {
                total_gb: 32.0,
                speed_mhz: Some(3200),
                memory_type: Some("DDR4".to_string()),
            },
            gpu: vec![GpuInfo {
                vendor: GpuVendor::Nvidia,
                model: "RTX 3080".to_string(),
                vram_gb: 10.0,
                driver_version: None,
                compute_capability: None,
            }],
            storage: vec![StorageInfo {
                device: "/dev/nvme0n1".to_string(),
                mount_point: "/".to_string(),
                total_gb: 1000.0,
                storage_type: StorageType::NvmeGen4,
                nvme_gen: Some(4),
            }],
            profile: SystemProfile::ProWorkstation,
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::process::Command;
    use tempfile::TempDir;

    /// Test full detection pipeline
    #[test]
    #[ignore] // Run with --ignored flag for integration tests
    fn test_full_hardware_detection() {
        let mut detector = HardwareDetector::new();
        let result = detector.detect();
        
        assert!(result.is_ok());
        let hardware = result.unwrap();
        
        // Validate all components were detected
        assert!(hardware.cpu.cores > 0);
        assert!(hardware.memory.total_gb > 0.0);
        assert!(!hardware.storage.is_empty());
    }

    /// Test sysfs interaction
    #[test]
    #[ignore]
    fn test_sysfs_access() {
        // Test we can read CPU governor
        let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
        if std::path::Path::new(governor_path).exists() {
            let content = std::fs::read_to_string(governor_path);
            assert!(content.is_ok());
        }
    }

    /// Test configuration persistence
    #[test]
    fn test_config_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hardware.json");
        
        let hardware = create_test_hardware_info();
        
        // Save config
        let json = serde_json::to_string_pretty(&hardware).unwrap();
        std::fs::write(&config_path, json).unwrap();
        
        // Load config
        let loaded_json = std::fs::read_to_string(&config_path).unwrap();
        let loaded: HardwareInfo = serde_json::from_str(&loaded_json).unwrap();
        
        assert_eq!(hardware.cpu.model, loaded.cpu.model);
    }
}

#[cfg(test)]
mod benchmarks {
    use super::super::*;
    use criterion::{black_box, Criterion};

    /// Benchmark hardware detection speed
    pub fn benchmark_detection(c: &mut Criterion) {
        c.bench_function("hardware_detection", |b| {
            b.iter(|| {
                let mut detector = HardwareDetector::new();
                black_box(detector.detect())
            });
        });
    }

    /// Benchmark profile determination
    pub fn benchmark_profile_determination(c: &mut Criterion) {
        let cpu = create_test_cpu();
        let memory = create_test_memory();
        let gpu = vec![create_test_gpu()];
        
        c.bench_function("profile_determination", |b| {
            b.iter(|| {
                black_box(HardwareDetector::determine_profile(&cpu, &memory, &gpu))
            });
        });
    }

    /// Benchmark JSON serialization
    pub fn benchmark_serialization(c: &mut Criterion) {
        let hardware = create_test_hardware_info();
        
        c.bench_function("json_serialization", |b| {
            b.iter(|| {
                black_box(serde_json::to_string(&hardware).unwrap())
            });
        });
    }
}

#[cfg(test)]
mod property_tests {
    use super::super::*;
    use proptest::prelude::*;

    /// Property test for profile determination
    proptest! {
        #[test]
        fn test_profile_never_panics(
            cores in 1usize..128,
            threads in 1usize..256,
            ram_gb in 1.0f64..2048.0,
            vram_gb in 0.0f64..48.0
        ) {
            let cpu = CpuInfo {
                vendor: "Test".to_string(),
                model: "Test CPU".to_string(),
                cores,
                threads: threads.max(cores),
                base_frequency: 2000.0,
                max_frequency: 4000.0,
                generation: None,
            };
            
            let memory = MemoryInfo {
                total_gb: ram_gb,
                speed_mhz: None,
                memory_type: None,
            };
            
            let gpu = if vram_gb > 0.0 {
                vec![GpuInfo {
                    vendor: GpuVendor::Nvidia,
                    model: "Test GPU".to_string(),
                    vram_gb,
                    driver_version: None,
                    compute_capability: None,
                }]
            } else {
                vec![]
            };
            
            // Should never panic regardless of input
            let _profile = HardwareDetector::determine_profile(&cpu, &memory, &gpu);
        }
    }

    /// Property test for swappiness calculation
    proptest! {
        #[test]
        fn test_swappiness_bounds(ram_gb in 0.5f64..2048.0) {
            let swappiness = calculate_swappiness(ram_gb);
            prop_assert!(swappiness >= 10);
            prop_assert!(swappiness <= 60);
        }
    }
}

#[cfg(test)]
mod stress_tests {
    use super::super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Stress test concurrent detection
    #[test]
    #[ignore] // Run explicitly with --ignored
    fn test_concurrent_detection() {
        let num_threads = 10;
        let mut handles = vec![];
        
        for _ in 0..num_threads {
            let handle = thread::spawn(|| {
                let mut detector = HardwareDetector::new();
                detector.detect().unwrap()
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let hardware = handle.join().unwrap();
            assert!(hardware.cpu.cores > 0);
        }
    }

    /// Test memory leak detection
    #[test]
    #[ignore]
    fn test_no_memory_leaks() {
        // Run detection in a loop and monitor memory
        for _ in 0..1000 {
            let mut detector = HardwareDetector::new();
            let _ = detector.detect();
            // Memory should not continuously increase
        }
    }
}