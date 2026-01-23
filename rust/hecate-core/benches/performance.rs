//! Performance benchmarks for HecateOS core components

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hecate_core::*;

/// Benchmark hardware detection performance
fn benchmark_hardware_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("hardware_detection");
    
    group.bench_function("full_detection", |b| {
        b.iter(|| {
            let mut detector = HardwareDetector::new();
            black_box(detector.detect())
        });
    });
    
    group.bench_function("cpu_only", |b| {
        b.iter(|| {
            let detector = HardwareDetector::new();
            black_box(detector.detect_cpu())
        });
    });
    
    group.bench_function("memory_only", |b| {
        b.iter(|| {
            let detector = HardwareDetector::new();
            black_box(detector.detect_memory())
        });
    });
    
    group.bench_function("storage_only", |b| {
        b.iter(|| {
            let detector = HardwareDetector::new();
            black_box(detector.detect_storage())
        });
    });
    
    group.finish();
}

/// Benchmark profile determination with different hardware configs
fn benchmark_profile_determination(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_determination");
    
    // Test different RAM sizes
    for ram_gb in [8.0, 16.0, 32.0, 64.0, 128.0] {
        group.bench_with_input(
            BenchmarkId::new("ram_gb", ram_gb),
            &ram_gb,
            |b, &ram| {
                let cpu = CpuInfo {
                    vendor: "Intel".to_string(),
                    model: "Test CPU".to_string(),
                    cores: 16,
                    threads: 32,
                    base_frequency: 3000.0,
                    max_frequency: 5000.0,
                    generation: Some(13),
                };
                
                let memory = MemoryInfo {
                    total_gb: ram,
                    speed_mhz: Some(5600),
                    memory_type: Some("DDR5".to_string()),
                };
                
                let gpu = vec![GpuInfo {
                    vendor: GpuVendor::Nvidia,
                    model: "RTX 4080".to_string(),
                    vram_gb: 16.0,
                    driver_version: None,
                    compute_capability: None,
                }];
                
                b.iter(|| {
                    black_box(HardwareDetector::determine_profile(&cpu, &memory, &gpu))
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark optimization application
fn benchmark_optimization_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_optimizations");
    
    for profile in [
        SystemProfile::AIFlagship,
        SystemProfile::ProWorkstation,
        SystemProfile::HighPerformance,
        SystemProfile::Developer,
        SystemProfile::Standard,
    ] {
        group.bench_with_input(
            BenchmarkId::new("profile", format!("{:?}", profile)),
            &profile,
            |b, &prof| {
                b.iter(|| {
                    black_box(apply_optimizations(&prof))
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark JSON serialization/deserialization
fn benchmark_serialization(c: &mut Criterion) {
    let hardware = HardwareInfo {
        cpu: CpuInfo {
            vendor: "Intel".to_string(),
            model: "Intel Core i9-13900K".to_string(),
            cores: 24,
            threads: 32,
            base_frequency: 3000.0,
            max_frequency: 5800.0,
            generation: Some(13),
        },
        memory: MemoryInfo {
            total_gb: 128.0,
            speed_mhz: Some(6400),
            memory_type: Some("DDR5".to_string()),
        },
        gpu: vec![
            GpuInfo {
                vendor: GpuVendor::Nvidia,
                model: "RTX 4090".to_string(),
                vram_gb: 24.0,
                driver_version: Some("545.29.06".to_string()),
                compute_capability: Some("8.9".to_string()),
            }
        ],
        storage: vec![
            StorageInfo {
                device: "/dev/nvme0n1".to_string(),
                mount_point: "/".to_string(),
                total_gb: 2000.0,
                storage_type: StorageType::NvmeGen4,
                nvme_gen: Some(4),
            }
        ],
        profile: SystemProfile::AIFlagship,
    };
    
    let mut group = c.benchmark_group("serialization");
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&hardware).unwrap())
        });
    });
    
    let json = serde_json::to_string(&hardware).unwrap();
    
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<HardwareInfo>(&json).unwrap())
        });
    });
    
    group.finish();
}

/// Benchmark storage type detection
fn benchmark_storage_detection(c: &mut Criterion) {
    c.bench_function("storage_type_from_name", |b| {
        let devices = vec!["nvme0n1", "nvme1n1", "sda", "sdb", "hda"];
        b.iter(|| {
            for device in &devices {
                black_box(HardwareDetector::detect_storage_type(device));
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_hardware_detection,
    benchmark_profile_determination,
    benchmark_optimization_application,
    benchmark_serialization,
    benchmark_storage_detection
);

criterion_main!(benches);