//! HecateOS Update System CLI
//!
//! Command-line interface for system update management

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use hecate_update::{UpdateManager, UpdateConfig, UpdateType, SecuritySeverity};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hecate-update")]
#[command(author, version, about = "HecateOS Intelligent Update System", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    /// Disable color output
    #[arg(long, global = true)]
    no_color: bool,
    
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Assume yes to all prompts
    #[arg(short, long, global = true)]
    yes: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for available updates
    Check {
        /// Show all updates including optional
        #[arg(short, long)]
        all: bool,
        
        /// Filter by update type
        #[arg(short = 't', long)]
        type_filter: Option<String>,
    },
    
    /// Apply system updates
    Apply {
        /// Update IDs to apply (all if empty)
        updates: Vec<String>,
        
        /// Skip creating snapshot before update
        #[arg(long)]
        no_snapshot: bool,
        
        /// Disable automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
        
        /// Force update even outside maintenance window
        #[arg(short, long)]
        force: bool,
    },
    
    /// Schedule updates for maintenance window
    Schedule {
        /// Update IDs to schedule
        updates: Vec<String>,
    },
    
    /// Rollback recent updates
    Rollback {
        /// Snapshot ID to rollback to
        snapshot: Option<String>,
    },
    
    /// Show update history
    History {
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Manage system snapshots
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },
    
    /// Configure update system
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    
    /// Show update system status
    Status,
    
    /// Run update service daemon
    Service {
        /// Run in foreground
        #[arg(short, long)]
        foreground: bool,
    },
}

#[derive(Subcommand)]
enum SnapshotAction {
    /// List available snapshots
    List,
    
    /// Create a new snapshot
    Create {
        /// Snapshot name
        name: Option<String>,
    },
    
    /// Delete a snapshot
    Delete {
        /// Snapshot ID
        id: String,
    },
    
    /// Show snapshot details
    Info {
        /// Snapshot ID
        id: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Enable live patching
    EnableLivePatch,
    
    /// Disable live patching
    DisableLivePatch,
    
    /// Enable automatic rollback
    EnableRollback,
    
    /// Disable automatic rollback
    DisableRollback,
    
    /// Set maintenance window
    SetMaintenanceWindow {
        /// Days of week (comma-separated)
        days: String,
        
        /// Start hour (0-23)
        start: u32,
        
        /// End hour (0-23)
        end: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .init();
    }
    
    // Set color output
    if cli.no_color {
        colored::control::set_override(false);
    }
    
    // Load configuration
    let config = if let Some(config_path) = cli.config {
        load_config(&config_path)?
    } else {
        UpdateConfig::default()
    };
    
    // Create update manager
    let mut manager = UpdateManager::new(config).await?;
    
    // Execute command
    match cli.command {
        Commands::Check { all, type_filter } => {
            handle_check(&mut manager, all, type_filter).await?;
        }
        Commands::Apply { updates, no_snapshot, no_rollback, force } => {
            handle_apply(&mut manager, updates, no_snapshot, no_rollback, force, cli.yes).await?;
        }
        Commands::Schedule { updates } => {
            handle_schedule(&mut manager, updates).await?;
        }
        Commands::Rollback { snapshot } => {
            handle_rollback(&mut manager, snapshot, cli.yes).await?;
        }
        Commands::History { limit, detailed } => {
            handle_history(&manager, limit, detailed).await?;
        }
        Commands::Snapshot { action } => {
            handle_snapshot(action).await?;
        }
        Commands::Config { action } => {
            handle_config(action).await?;
        }
        Commands::Status => {
            handle_status(&manager).await?;
        }
        Commands::Service { foreground } => {
            handle_service(foreground).await?;
        }
    }
    
    Ok(())
}

async fn handle_check(
    manager: &mut UpdateManager,
    show_all: bool,
    type_filter: Option<String>,
) -> Result<()> {
    println!("{}", "Checking for system updates...".bright_cyan());
    
    let updates = manager.check_updates().await?;
    
    if updates.is_empty() {
        println!("{}", "System is up to date!".green());
        return Ok(());
    }
    
    // Filter updates
    let filtered: Vec<_> = updates.iter()
        .filter(|u| {
            if let Some(ref filter) = type_filter {
                match filter.as_str() {
                    "kernel" => matches!(u.update_type, UpdateType::KernelPatch { .. }),
                    "driver" => matches!(u.update_type, UpdateType::Driver { .. }),
                    "package" => matches!(u.update_type, UpdateType::Package { .. }),
                    "firmware" => matches!(u.update_type, UpdateType::Firmware { .. }),
                    "security" => matches!(u.update_type, UpdateType::Security { .. }),
                    _ => true,
                }
            } else {
                true
            }
        })
        .collect();
    
    println!("\n{}", format!("Found {} updates:", filtered.len()).bright_yellow());
    
    for update in filtered {
        let type_str = match &update.update_type {
            UpdateType::KernelPatch { version, .. } => {
                format!("Kernel {}", version).bright_blue()
            }
            UpdateType::Driver { name, version, .. } => {
                format!("Driver {}-{}", name, version).bright_magenta()
            }
            UpdateType::Package { name, version } => {
                format!("Package {}-{}", name, version).bright_white()
            }
            UpdateType::Firmware { component, version, .. } => {
                format!("Firmware {}-{}", component, version).bright_cyan()
            }
            UpdateType::Security { cve_id, severity, .. } => {
                let color = match severity {
                    SecuritySeverity::Critical => format!("Security {} (CRITICAL)", cve_id).red().bold(),
                    SecuritySeverity::High => format!("Security {} (HIGH)", cve_id).red(),
                    SecuritySeverity::Medium => format!("Security {} (MEDIUM)", cve_id).yellow(),
                    SecuritySeverity::Low => format!("Security {} (LOW)", cve_id).bright_black(),
                };
                color
            }
        };
        
        println!("  [{}] {} - {}", 
            update.id.bright_black(),
            type_str,
            update.description
        );
        
        let size_mb = update.size_bytes as f64 / (1024.0 * 1024.0);
        println!("      {} {:.1} MB", 
            "Size:".bright_black(),
            size_mb
        );
        
        if show_all {
            if !update.dependencies.is_empty() {
                println!("      {} {}", 
                    "Depends:".bright_black(),
                    update.dependencies.join(", ")
                );
            }
        }
    }
    
    // Show summary
    let critical_count = updates.iter()
        .filter(|u| matches!(u.update_type, 
            UpdateType::Security { severity: SecuritySeverity::Critical, .. }))
        .count();
    
    if critical_count > 0 {
        println!("\n{}", 
            format!("⚠ {} critical security updates available!", critical_count)
                .red().bold()
        );
    }
    
    let total_size: u64 = updates.iter().map(|u| u.size_bytes).sum();
    let total_size_mb = total_size as f64 / (1024.0 * 1024.0);
    
    println!("\n{} {:.1} MB", 
        "Total download size:".bright_black(),
        total_size_mb
    );
    
    Ok(())
}

async fn handle_apply(
    manager: &mut UpdateManager,
    update_ids: Vec<String>,
    no_snapshot: bool,
    no_rollback: bool,
    force: bool,
    auto_yes: bool,
) -> Result<()> {
    // Get available updates
    let available = manager.check_updates().await?;
    
    // Determine which updates to apply
    let to_apply = if update_ids.is_empty() {
        // Apply all available
        available.iter().map(|u| u.id.clone()).collect()
    } else {
        update_ids
    };
    
    if to_apply.is_empty() {
        println!("{}", "No updates to apply".yellow());
        return Ok(());
    }
    
    // Create update plan
    let mut plan = manager.create_plan(to_apply).await?;
    
    // Modify plan based on flags
    if no_snapshot {
        plan.snapshot_before = false;
    }
    if no_rollback {
        plan.auto_rollback = false;
    }
    
    // Show plan
    println!("\n{}", "Update Plan:".bright_cyan());
    println!("  Updates to apply: {}", plan.updates.len());
    println!("  Estimated time: {:?}", plan.estimated_time);
    println!("  Requires reboot: {}", 
        if plan.requires_reboot { "Yes".red() } else { "No".green() }
    );
    println!("  Create snapshot: {}", 
        if plan.snapshot_before { "Yes".green() } else { "No".yellow() }
    );
    println!("  Auto-rollback: {}", 
        if plan.auto_rollback { "Yes".green() } else { "No".yellow() }
    );
    
    // Confirm
    if !auto_yes {
        let confirm = Confirm::new()
            .with_prompt("Proceed with update?")
            .default(true)
            .interact()?;
        
        if !confirm {
            println!("{}", "Update cancelled".yellow());
            return Ok(());
        }
    }
    
    // Apply updates
    println!("\n{}", "Applying updates...".bright_cyan());
    
    let pb = ProgressBar::new(plan.updates.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("##-"),
    );
    
    match manager.apply_updates(plan).await {
        Ok(()) => {
            pb.finish_with_message("✓ All updates applied successfully");
            println!("\n{}", "Updates applied successfully!".green().bold());
        }
        Err(e) => {
            pb.finish_with_message("✗ Update failed");
            eprintln!("\n{}: {}", "Update failed".red().bold(), e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn handle_schedule(
    manager: &mut UpdateManager,
    update_ids: Vec<String>,
) -> Result<()> {
    if update_ids.is_empty() {
        println!("{}", "No updates specified".yellow());
        return Ok(());
    }
    
    manager.schedule_updates(update_ids).await?;
    println!("{}", "Updates scheduled for next maintenance window".green());
    
    Ok(())
}

async fn handle_rollback(
    manager: &mut UpdateManager,
    snapshot: Option<String>,
    auto_yes: bool,
) -> Result<()> {
    println!("{}", "⚠ WARNING: This will rollback recent system changes!".red().bold());
    
    if !auto_yes {
        let confirm = Confirm::new()
            .with_prompt("Are you sure you want to rollback?")
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("{}", "Rollback cancelled".yellow());
            return Ok(());
        }
    }
    
    println!("{}", "Initiating rollback...".bright_cyan());
    manager.rollback().await?;
    println!("{}", "Rollback completed successfully!".green().bold());
    
    Ok(())
}

async fn handle_history(
    manager: &UpdateManager,
    limit: usize,
    detailed: bool,
) -> Result<()> {
    let history = manager.get_history().await?;
    
    if history.is_empty() {
        println!("{}", "No update history".yellow());
        return Ok(());
    }
    
    println!("{}", "Update History:".bright_cyan());
    
    for (i, entry) in history.iter().take(limit).enumerate() {
        println!("\n{}. [{}] {}", 
            i + 1,
            entry.id.bright_black(),
            entry.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
        
        let status_str = match &entry.status {
            hecate_update::UpdateStatus::Installed => "Installed".green(),
            hecate_update::UpdateStatus::Failed { error } => format!("Failed: {}", error).red(),
            hecate_update::UpdateStatus::RolledBack => "Rolled Back".yellow(),
            _ => format!("{:?}", entry.status).normal(),
        };
        
        println!("   Status: {}", status_str);
        println!("   Duration: {:?}", entry.duration);
        
        if detailed {
            // Show more details
        }
    }
    
    Ok(())
}

async fn handle_snapshot(action: SnapshotAction) -> Result<()> {
    // TODO: Implement snapshot management
    match action {
        SnapshotAction::List => {
            println!("Available snapshots:");
        }
        SnapshotAction::Create { name } => {
            println!("Creating snapshot...");
        }
        SnapshotAction::Delete { id } => {
            println!("Deleting snapshot {}...", id);
        }
        SnapshotAction::Info { id } => {
            println!("Snapshot {} info:", id);
        }
    }
    Ok(())
}

async fn handle_config(action: ConfigAction) -> Result<()> {
    // TODO: Implement configuration management
    match action {
        ConfigAction::Show => {
            println!("Current configuration:");
        }
        ConfigAction::Set { key, value } => {
            println!("Setting {} = {}", key, value);
        }
        ConfigAction::EnableLivePatch => {
            println!("Live patching enabled");
        }
        ConfigAction::DisableLivePatch => {
            println!("Live patching disabled");
        }
        ConfigAction::EnableRollback => {
            println!("Automatic rollback enabled");
        }
        ConfigAction::DisableRollback => {
            println!("Automatic rollback disabled");
        }
        ConfigAction::SetMaintenanceWindow { days, start, end } => {
            println!("Maintenance window set: {} from {}:00 to {}:00", days, start, end);
        }
    }
    Ok(())
}

async fn handle_status(manager: &UpdateManager) -> Result<()> {
    println!("{}", "=== Update System Status ===".bright_cyan().bold());
    
    // TODO: Show actual status
    println!("\nLive Patching: {}", "Enabled".green());
    println!("Hot Swapping: {}", "Enabled".green());
    println!("Auto Rollback: {}", "Enabled".green());
    println!("\nMaintenance Window: {} 02:00-06:00", "Sun, Wed".bright_white());
    println!("Next Window: {}", "2025-02-05 02:00:00".bright_white());
    
    Ok(())
}

async fn handle_service(foreground: bool) -> Result<()> {
    println!("{}", "Starting update service...".bright_cyan());
    
    if !foreground {
        // TODO: Daemonize process
        println!("Running in background");
    } else {
        println!("Running in foreground (Ctrl+C to stop)");
        // TODO: Run service loop
    }
    
    Ok(())
}

fn load_config(path: &PathBuf) -> Result<UpdateConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: UpdateConfig = toml::from_str(&content)?;
    Ok(config)
}