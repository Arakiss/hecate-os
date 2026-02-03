use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about = "HecateOS architecture validator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate project structure
    Validate,
    /// Check for circular dependencies
    Cycles,
    /// Show module boundaries
    Boundaries,
    /// Validate port configuration
    Ports,
    /// Generate architecture diagram
    Diagram,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Validate => validate_structure()?,
        Commands::Cycles => check_cycles()?,
        Commands::Boundaries => show_boundaries()?,
        Commands::Ports => validate_ports()?,
        Commands::Diagram => generate_diagram()?,
    }
    
    Ok(())
}

fn validate_structure() -> Result<()> {
    println!("{} Validating architecture...", "→".blue());
    
    let required_structure = vec![
        ("rust/hecate-core", "Core library"),
        ("rust/hecate-daemon", "System daemon"),
        ("rust/hecate-gpu", "GPU management"),
        ("rust/hecate-pkg", "Package manager"),
        ("rust/hecate-dev", "Development tools"),
        ("hecate-dashboard", "Web dashboard"),
        ("docs", "Documentation"),
        ("scripts", "System scripts"),
        ("config", "Configuration"),
    ];
    
    let mut all_valid = true;
    
    for (path, description) in required_structure {
        if Path::new(path).exists() {
            println!("  {} {} - {}", "✓".green(), path, description.dimmed());
        } else {
            println!("  {} {} - {} {}", "✗".red(), path, description.dimmed(), "MISSING".red());
            all_valid = false;
        }
    }
    
    if all_valid {
        println!("\n{} Architecture structure is valid", "✓".green().bold());
    } else {
        anyhow::bail!("Architecture validation failed");
    }
    
    Ok(())
}

fn check_cycles() -> Result<()> {
    println!("{} Checking for circular dependencies...", "→".blue());
    
    let mut graph = DiGraph::new();
    let mut nodes: HashMap<String, NodeIndex> = HashMap::new();
    
    // Parse Cargo.toml files to build dependency graph
    for entry in WalkDir::new("rust")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml")
    {
        let content = fs::read_to_string(entry.path())?;
        if let Ok(doc) = content.parse::<toml_edit::DocumentMut>() {
            if let Some(package) = doc.get("package").and_then(|p| p.get("name")) {
                let package_name = package.as_str().unwrap_or("").to_string();
                
                if !nodes.contains_key(&package_name) {
                    let idx = graph.add_node(package_name.clone());
                    nodes.insert(package_name.clone(), idx);
                }
                
                // Check dependencies
                if let Some(deps) = doc.get("dependencies") {
                    if let Some(table) = deps.as_table() {
                        for (dep_name, _) in table {
                            if dep_name.starts_with("hecate-") {
                                if !nodes.contains_key(dep_name) {
                                    let idx = graph.add_node(dep_name.to_string());
                                    nodes.insert(dep_name.to_string(), idx);
                                }
                                
                                let from = nodes[&package_name];
                                let to = nodes[dep_name];
                                graph.add_edge(from, to, ());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Check for cycles using Tarjan's algorithm
    if petgraph::algo::is_cyclic_directed(&graph) {
        println!("{} Circular dependencies detected!", "✗".red().bold());
        anyhow::bail!("Circular dependencies found in module graph");
    } else {
        println!("{} No circular dependencies found", "✓".green().bold());
    }
    
    Ok(())
}

fn show_boundaries() -> Result<()> {
    println!("{} Module boundaries:", "→".blue());
    
    println!("\n{}", "Layer Architecture:".bold());
    println!("
┌─────────────────────────────────────┐
│         Applications Layer          │
│  (hecate-dashboard, hecate-cli)    │
├─────────────────────────────────────┤
│         Services Layer              │
│  (hecate-daemon, hecate-monitor)   │
├─────────────────────────────────────┤
│         Domain Layer                │
│  (hecate-gpu, hecate-pkg)          │
├─────────────────────────────────────┤
│         Core Layer                  │
│        (hecate-core)                │
└─────────────────────────────────────┘
");
    
    println!("{}", "Rules:".bold());
    println!("  • Dependencies flow downward only");
    println!("  • Core has no dependencies on other modules");
    println!("  • Services can depend on Domain and Core");
    println!("  • Applications can depend on all layers");
    
    Ok(())
}

fn validate_ports() -> Result<()> {
    println!("{} Validating port configuration...", "→".blue());
    
    let expected_ports = vec![
        ("MONITOR", 9313, "WebSocket monitoring"),
        ("PKG_API", 9314, "Package manager API"),
        ("REMOTE", 9315, "Remote management"),
        ("BENCH", 9316, "Benchmark server"),
        ("GPU", 9317, "GPU management"),
    ];
    
    let config_path = "config/hecate/ports.conf";
    let config = if Path::new(config_path).exists() {
        fs::read_to_string(config_path)?
    } else {
        String::new()
    };
    
    let mut all_found = true;
    let mut used_ports = HashSet::new();
    
    for (name, port, description) in &expected_ports {
        let pattern = format!("{}={}", name, port);
        if config.contains(&pattern) {
            println!("  {} Port {} ({}) - {}", "✓".green(), port, name, description.dimmed());
            
            if !used_ports.insert(port) {
                println!("    {} Duplicate port detected!", "⚠".yellow());
                all_found = false;
            }
        } else {
            println!("  {} Port {} ({}) - {} {}", "✗".red(), port, name, description.dimmed(), "NOT CONFIGURED".red());
            all_found = false;
        }
    }
    
    if all_found {
        println!("\n{} Port configuration is valid", "✓".green().bold());
    } else {
        anyhow::bail!("Port configuration issues detected");
    }
    
    Ok(())
}

fn generate_diagram() -> Result<()> {
    println!("{} Generating architecture diagram...", "→".blue());
    
    let diagram = r#"
HecateOS Architecture
=====================

┌──────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│                                                              │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │ hecate-dashboard │        │   hecate-cli     │          │
│  │    (Next.js)     │        │     (Rust)       │          │
│  └────────┬─────────┘        └────────┬─────────┘          │
└───────────┼────────────────────────────┼────────────────────┘
            │                            │
            │         WebSocket          │
            │          (9313)            │
            ▼                            ▼
┌──────────────────────────────────────────────────────────────┐
│                     System Services                          │
│                                                              │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │  hecate-monitor  │        │  hecate-daemon   │          │
│  │   (Real-time)    │        │   (Optimizer)    │          │
│  └────────┬─────────┘        └────────┬─────────┘          │
└───────────┼────────────────────────────┼────────────────────┘
            │                            │
            ▼                            ▼
┌──────────────────────────────────────────────────────────────┐
│                      Core Modules                            │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  hecate-gpu  │  │  hecate-pkg  │  │ hecate-bench │     │
│  │   (NVIDIA)   │  │   (Packages) │  │ (Benchmarks) │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
└─────────┼──────────────────┼──────────────────┼─────────────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                            ▼
                  ┌──────────────────┐
                  │   hecate-core    │
                  │  (Shared Types)  │
                  └──────────────────┘
"#;
    
    println!("{}", diagram);
    
    let output_path = "docs/architecture-diagram.txt";
    fs::write(output_path, diagram)?;
    println!("\n{} Diagram saved to {}", "✓".green().bold(), output_path);
    
    Ok(())
}