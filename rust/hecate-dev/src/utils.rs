use anyhow::{Context, Result};
use colored::*;
use std::process::Command;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: &'static str,
    pub command: &'static str,
    pub package: &'static str,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug)]
pub struct DependencyStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// Check system dependencies
pub fn check_dependencies() -> Result<HashMap<String, DependencyStatus>> {
    let dependencies = vec![
        Dependency {
            name: "cargo",
            command: "cargo",
            package: "cargo",
            required: true,
            description: "Rust build tool",
        },
        Dependency {
            name: "git",
            command: "git",
            package: "git",
            required: true,
            description: "Version control",
        },
        Dependency {
            name: "7z",
            command: "7z",
            package: "p7zip-full",
            required: false,
            description: "Faster ISO extraction (optional)",
        },
    ];

    let mut results = HashMap::new();
    
    for dep in dependencies {
        let status = check_command(dep.command);
        results.insert(dep.name.to_string(), status);
    }
    
    Ok(results)
}

fn check_command(cmd: &str) -> DependencyStatus {
    // Check if command exists
    let which_result = Command::new("which")
        .arg(cmd)
        .output();
    
    let (installed, path) = if let Ok(output) = which_result {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(path))
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };
    
    // Try to get version if installed
    let version = if installed {
        let version_result = Command::new(cmd)
            .arg("--version")
            .output();
        
        if let Ok(output) = version_result {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string())
            } else {
                // Some tools use -v instead
                let version_result = Command::new(cmd)
                    .arg("-v")
                    .output();
                
                if let Ok(output) = version_result {
                    Some(String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string())
                } else {
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };
    
    DependencyStatus {
        installed,
        version,
        path,
    }
}

/// Print dependency check results
pub fn print_dependency_report(results: &HashMap<String, DependencyStatus>) {
    println!("\n{}", "System Dependencies".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_cyan());
    
    let dependencies = vec![
        ("cargo", "Rust build tool", "cargo", true),
        ("git", "Version control", "git", true),
        ("7z", "ISO extraction (faster alternative)", "p7zip-full", false),
    ];
    
    let mut missing_required = Vec::new();
    let mut missing_optional = Vec::new();
    
    for (name, desc, package, required) in dependencies {
        if let Some(status) = results.get(name) {
            let icon = if status.installed {
                "✅".green()
            } else if required {
                "❌".red()
            } else {
                "⚠️ ".yellow()
            };
            
            let status_text = if status.installed {
                if let Some(ref version) = status.version {
                    format!("{} {}", "Installed".green(), version.bright_black())
                } else {
                    "Installed".green().to_string()
                }
            } else {
                format!("{} (install: sudo apt-get install {})", 
                    "Not found".red(), 
                    package.yellow())
            };
            
            println!("  {} {:<12} {} - {}", 
                icon, 
                name.bright_white(), 
                status_text,
                desc.bright_black());
            
            if !status.installed {
                if required {
                    missing_required.push((name, package));
                } else {
                    missing_optional.push((name, package));
                }
            }
        }
    }
    
    // Print installation instructions
    if !missing_required.is_empty() {
        println!("\n{} Required dependencies missing!", "ERROR:".red().bold());
        println!("Install with:");
        println!("  {}", format!("sudo apt-get install {}", 
            missing_required.iter()
                .map(|(_, p)| *p)
                .collect::<Vec<_>>()
                .join(" "))
            .bright_yellow());
    }
    
    if !missing_optional.is_empty() {
        println!("\n{} Optional tools for enhanced performance:", "INFO:".blue());
        println!("  7z provides faster ISO extraction than our native implementation");
        println!("  Install if needed: {}", 
            format!("sudo apt-get install {}", 
                missing_optional.iter()
                    .map(|(_, p)| *p)
                    .collect::<Vec<_>>()
                    .join(" "))
            .bright_black());
    }
    
    // Check for at least one ISO extraction tool
    let has_extraction = results.get("7z").map_or(false, |s| s.installed) ||
                        results.get("bsdtar").map_or(false, |s| s.installed) ||
                        results.get("xorriso").map_or(false, |s| s.installed);
    
    if !has_extraction {
        println!("\n{} No ISO extraction tool found!", "WARNING:".yellow().bold());
        println!("You need at least one of these to create ISOs:");
        println!("  {} (recommended)", "sudo apt-get install p7zip-full".bright_yellow());
    }
}

/// Check if we have at least one ISO extraction tool
pub fn has_iso_extraction_tool() -> bool {
    check_command("7z").installed || 
    check_command("bsdtar").installed || 
    check_command("xorriso").installed
}

/// Check if we have ISO creation capability (always true with native implementation)
pub fn has_iso_creation_tool() -> bool {
    true // We have native Rust ISO creation!
}

/// Show a nice error message with installation instructions
pub fn show_missing_tool_error(tool_type: &str) -> Result<()> {
    match tool_type {
        "extraction" => {
            eprintln!("\n{}", "═".repeat(60).red());
            eprintln!("{}", "❌ ISO Extraction Tool Required".red().bold());
            eprintln!("{}", "═".repeat(60).red());
            eprintln!("\nNo ISO extraction tool found. Please install one:");
            eprintln!("");
            eprintln!("  {} {}", "Recommended:".green(), "sudo apt-get install p7zip-full".bright_yellow());
            eprintln!("  {} {}", "Alternative:".bright_black(), "sudo apt-get install libarchive-tools");
            eprintln!("  {} {}", "Alternative:".bright_black(), "sudo apt-get install xorriso");
            eprintln!("");
            eprintln!("After installation, run this command again.");
            eprintln!("{}", "═".repeat(60).red());
            Err(anyhow::anyhow!("Missing required tool"))
        }
        "creation" => {
            eprintln!("\n{}", "═".repeat(60).red());
            eprintln!("{}", "❌ ISO Creation Tool Required".red().bold());
            eprintln!("{}", "═".repeat(60).red());
            eprintln!("\nNo ISO creation tool found. Please install one:");
            eprintln!("");
            eprintln!("  {} {}", "Recommended:".green(), "sudo apt-get install xorriso".bright_yellow());
            eprintln!("  {} {}", "Alternative:".bright_black(), "sudo apt-get install genisoimage");
            eprintln!("");
            eprintln!("After installation, run this command again.");
            eprintln!("{}", "═".repeat(60).red());
            Err(anyhow::anyhow!("Missing required tool"))
        }
        _ => Ok(())
    }
}

/// Print a progress message with nice formatting
pub fn progress_msg(icon: &str, message: &str) {
    println!("  {} {}", icon, message);
}

/// Print a success message
pub fn success_msg(message: &str) {
    println!("  {} {}", "✅".green(), message.green());
}

/// Print an error message
pub fn error_msg(message: &str) {
    eprintln!("  {} {}", "❌".red(), message.red());
}

/// Print a warning message
pub fn warn_msg(message: &str) {
    println!("  {} {}", "⚠️".yellow(), message.yellow());
}

/// Print a info message
pub fn info_msg(message: &str) {
    println!("  {} {}", "ℹ️".blue(), message.bright_blue());
}

/// Print a header
pub fn print_header(title: &str) {
    println!("\n{}", title.bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
}