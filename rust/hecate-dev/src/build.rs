use anyhow::{Context, Result};
use std::process::Command;
use std::path::PathBuf;
use tracing::{info, warn, error};
use colored::*;

const COMPONENTS: &[&str] = &[
    "hecate-cli",
    "hecate-daemon",
    "hecate-monitor", 
    "hecate-bench",
    "hecate-pkg",
    "hecate-gpu",
    "hecate-ml",
    "hecate-dev",
    "hecate-sign",
    "hecate-iso-builder",
];

pub async fn build_all(release: bool, run_tests: bool) -> Result<()> {
    println!("{}", "Building all HecateOS components...".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_cyan());
    
    let rust_dir = find_project_root()?;
    let mode = if release { "release" } else { "debug" };
    
    let mut failed = Vec::new();
    let total = COMPONENTS.len();
    
    for (idx, component) in COMPONENTS.iter().enumerate() {
        println!("\n[{}/{}] Building {}...", idx + 1, total, component.bright_yellow());
        
        if let Err(e) = build_single_component(&rust_dir, component, release) {
            error!("Failed to build {}: {}", component, e);
            failed.push(*component);
        } else {
            println!("  ✅ {} built successfully", component.green());
        }
    }
    
    if run_tests {
        println!("\n{}", "Running tests...".bright_cyan());
        run_all_tests(&rust_dir)?;
    }
    
    if !failed.is_empty() {
        println!("\n{}", "Build Summary:".bright_red());
        println!("  Failed components: {:?}", failed);
        return Err(anyhow::anyhow!("Some components failed to build"));
    }
    
    println!("\n{}", "✅ All components built successfully!".green().bold());
    println!("  Mode: {}", mode.bright_yellow());
    println!("  Output: {}/target/{}/", rust_dir.display(), mode);
    
    Ok(())
}

pub async fn build_component(name: &str, release: bool) -> Result<()> {
    let rust_dir = find_project_root()?;
    
    if !COMPONENTS.contains(&name) {
        return Err(anyhow::anyhow!(
            "Unknown component: {}. Available: {:?}", 
            name, 
            COMPONENTS
        ));
    }
    
    println!("Building {}...", name.bright_yellow());
    build_single_component(&rust_dir, name, release)?;
    println!("✅ {} built successfully", name.green());
    
    Ok(())
}

fn build_single_component(rust_dir: &PathBuf, component: &str, release: bool) -> Result<()> {
    let component_dir = rust_dir.join(component);
    
    if !component_dir.exists() {
        return Err(anyhow::anyhow!("Component directory not found: {}", component_dir.display()));
    }
    
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&component_dir)
        .arg("build");
    
    if release {
        cmd.arg("--release");
    }
    
    // Special handling for hecate-pkg
    if component == "hecate-pkg" {
        cmd.env("DATABASE_URL", "sqlite:hecate-pkg.db");
    }
    
    let output = cmd.output()
        .context(format!("Failed to run cargo build for {}", component))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Build failed:\n{}", stderr));
    }
    
    Ok(())
}

pub async fn clean(deep: bool) -> Result<()> {
    let rust_dir = find_project_root()?;
    
    println!("Cleaning build artifacts...");
    
    if deep {
        // Remove all target directories
        for component in COMPONENTS {
            let target_dir = rust_dir.join(component).join("target");
            if target_dir.exists() {
                println!("  Removing {}/target", component);
                std::fs::remove_dir_all(&target_dir)?;
            }
        }
        
        // Remove root target directory
        let root_target = rust_dir.join("target");
        if root_target.exists() {
            println!("  Removing root target directory");
            std::fs::remove_dir_all(&root_target)?;
        }
    } else {
        // Just run cargo clean
        Command::new("cargo")
            .current_dir(&rust_dir)
            .arg("clean")
            .status()?;
    }
    
    println!("✅ Clean complete");
    Ok(())
}

pub async fn show_status() -> Result<()> {
    let rust_dir = find_project_root()?;
    
    println!("{}", "HecateOS Build Status".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_cyan());
    
    let release_dir = rust_dir.join("target/release");
    let debug_dir = rust_dir.join("target/debug");
    
    // Map component names to their actual binary names (if they produce binaries)
    let binary_components = [
        ("hecate-cli", Some("hecate")),
        ("hecate-daemon", Some("hecated")),
        ("hecate-monitor", Some("hecate-monitor")),
        ("hecate-bench", Some("hecate-bench")),
        ("hecate-pkg", Some("hecate-pkg")),
        ("hecate-gpu", None),  // Library crate, no binary
        ("hecate-ml", None),   // Library crate, no binary
        ("hecate-dev", Some("hecate-dev")),
        ("hecate-sign", Some("hecate-sign")),
        ("hecate-iso-builder", Some("hecate-iso")),
    ];
    
    for (component, binary_name) in binary_components {
        let status = if let Some(binary) = binary_name {
            let release_bin = release_dir.join(binary);
            let debug_bin = debug_dir.join(binary);
            
            if release_bin.exists() {
                "✅ Release".green()
            } else if debug_bin.exists() {
                "🔧 Debug".yellow()
            } else {
                "❌ Not built".red()
            }
        } else {
            // For library crates, check if they were built by looking for .rlib files  
            let component_clean = component.replace("-", "_");
            
            // Check if the release or debug deps directory contains the library
            let release_deps = rust_dir.join("target/release/deps");
            let debug_deps = rust_dir.join("target/debug/deps");
            
            let has_release = if release_deps.exists() {
                std::fs::read_dir(&release_deps)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok())
                            .any(|e| {
                                let name = e.file_name();
                                let name_str = name.to_string_lossy();
                                name_str.starts_with(&format!("lib{}", component_clean)) && 
                                (name_str.ends_with(".rlib") || name_str.ends_with(".rmeta"))
                            })
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            
            let has_debug = if debug_deps.exists() {
                std::fs::read_dir(&debug_deps)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok())
                            .any(|e| {
                                let name = e.file_name();
                                let name_str = name.to_string_lossy();
                                name_str.starts_with(&format!("lib{}", component_clean)) &&
                                (name_str.ends_with(".rlib") || name_str.ends_with(".rmeta"))
                            })
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            
            if has_release {
                "✅ Release (lib)".green()
            } else if has_debug {
                "🔧 Debug (lib)".yellow()
            } else {
                "❌ Not built".red()
            }
        };
        
        println!("  {:<20} {}", component, status);
    }
    
    // Check for uncommitted changes
    let output = Command::new("git")
        .current_dir(&rust_dir)
        .args(&["status", "--porcelain"])
        .output()?;
    
    if !output.stdout.is_empty() {
        println!("\n⚠️  {} Uncommitted changes detected", "Warning:".yellow());
    }
    
    Ok(())
}

fn run_all_tests(rust_dir: &PathBuf) -> Result<()> {
    let output = Command::new("cargo")
        .current_dir(rust_dir)
        .arg("test")
        .arg("--all")
        .arg("--quiet")
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Tests failed:\n{}", stderr));
    }
    
    println!("✅ All tests passed");
    Ok(())
}

pub fn find_project_root() -> Result<PathBuf> {
    // First check if we're already in the rust directory
    let current = std::env::current_dir()?;
    if current.join("Makefile").exists() && current.join("hecate-daemon").is_dir() {
        return Ok(current);
    }
    
    // Check if HECATE_ROOT env var is set
    if let Ok(root) = std::env::var("HECATE_ROOT") {
        let root_path = PathBuf::from(root);
        if root_path.join("Makefile").exists() && root_path.join("hecate-daemon").is_dir() {
            return Ok(root_path);
        }
    }
    
    // Try to find based on the executable location
    if let Ok(exe_path) = std::env::current_exe() {
        // Check if we're in target/release or target/debug
        if let Some(parent) = exe_path.parent() {
            if let Some(target) = parent.parent() {
                if let Some(rust_dir) = target.parent() {
                    if rust_dir.join("Makefile").exists() && rust_dir.join("hecate-daemon").is_dir() {
                        return Ok(rust_dir.to_path_buf());
                    }
                }
            }
            
            // Check if installed in ~/.local/bin and find project
            if parent.ends_with(".local/bin") {
                // Look for common project locations
                let home = std::env::var("HOME").unwrap_or_default();
                let possible_paths = vec![
                    PathBuf::from(&home).join("Projects/personal/hecate-os/rust"),
                    PathBuf::from(&home).join("projects/hecate-os/rust"),
                    PathBuf::from(&home).join("hecate-os/rust"),
                    PathBuf::from("/home/akkarin/Projects/personal/hecate-os/rust"),
                ];
                
                for path in possible_paths {
                    if path.join("Makefile").exists() && path.join("hecate-daemon").is_dir() {
                        return Ok(path);
                    }
                }
            }
        }
    }
    
    // Try searching upward from current directory
    let mut search_dir = current.clone();
    for _ in 0..5 {
        if search_dir.join("rust/Makefile").exists() && search_dir.join("rust/hecate-daemon").is_dir() {
            return Ok(search_dir.join("rust"));
        }
        if let Some(parent) = search_dir.parent() {
            search_dir = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    Err(anyhow::anyhow!(
        "Could not find HecateOS project root. Set HECATE_ROOT environment variable to /path/to/hecate-os/rust"
    ))
}