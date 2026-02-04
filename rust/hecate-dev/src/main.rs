use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::{info, warn};
use tracing_subscriber::filter::EnvFilter;

mod version;
mod commit;
mod check;
mod release;
mod build;
mod iso;
mod utils;
mod setup;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage project versions
    Version {
        #[command(subcommand)]
        action: VersionAction,
    },
    /// Validate and format commits
    Commit {
        #[command(subcommand)]
        action: CommitAction,
    },
    /// Check project structure and conventions
    Check {
        /// Check only specific aspects
        #[arg(short, long)]
        only: Option<Vec<String>>,
        
        /// Auto-fix issues when possible
        #[arg(short, long)]
        fix: bool,
    },
    /// Manage releases
    Release {
        #[command(subcommand)]
        action: ReleaseAction,
    },
    /// Initialize git hooks
    InitHooks {
        /// Force overwrite existing hooks
        #[arg(short, long)]
        force: bool,
    },
    /// Build HecateOS components
    Build {
        #[command(subcommand)]
        action: BuildAction,
    },
    /// Create HecateOS ISO
    Iso {
        #[command(subcommand)]
        action: IsoAction,
    },
    /// Check system dependencies and configuration
    Doctor {
        /// Fix common issues automatically
        #[arg(long)]
        fix: bool,
    },
    /// Setup development environment
    Setup {
        /// Automatically install missing tools
        #[arg(long)]
        auto: bool,
    },
}

#[derive(Subcommand)]
enum VersionAction {
    /// Show current version
    Show,
    /// Bump version based on commit type
    Bump {
        /// Version part to bump (major, minor, patch)
        #[arg(value_enum)]
        level: version::BumpLevel,
        
        /// Dry run without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Sync version across all files
    Sync {
        /// Version to set
        version: Option<String>,
    },
    /// Check if versions are in sync
    Check,
}

#[derive(Subcommand)]
enum CommitAction {
    /// Validate commit message format
    Validate {
        /// Commit message or range
        message: Option<String>,
    },
    /// Create a properly formatted commit
    Create {
        /// Commit type (feat, fix, chore, etc.)
        #[arg(short = 't', long)]
        commit_type: String,
        
        /// Scope of the change
        #[arg(short, long)]
        scope: Option<String>,
        
        /// Commit message
        #[arg(short, long)]
        message: String,
        
        /// Breaking change
        #[arg(short, long)]
        breaking: bool,
    },
    /// Show commit conventions
    Conventions,
}

#[derive(Subcommand)]
enum BuildAction {
    /// Build all components
    All {
        /// Build in release mode
        #[arg(short, long, default_value = "true")]
        release: bool,
        
        /// Run tests after building
        #[arg(short, long)]
        test: bool,
    },
    /// Build specific component
    Component {
        /// Component name
        name: String,
        
        /// Build in release mode
        #[arg(short, long, default_value = "true")]
        release: bool,
    },
    /// Clean build artifacts
    Clean {
        /// Remove target directories
        #[arg(long)]
        deep: bool,
    },
    /// Show build status
    Status,
}

#[derive(Subcommand)]
enum IsoAction {
    /// Create HecateOS ISO
    Create {
        /// Ubuntu version to download (24.04 or 22.04)
        #[arg(short, long)]
        download: Option<String>,
        
        /// Input ISO file (if not downloading)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output ISO file
        #[arg(short, long, default_value = "hecateos.iso")]
        output: String,
        
        /// Skip building components
        #[arg(long)]
        skip_build: bool,
        
        /// Include source code
        #[arg(long)]
        with_source: bool,
    },
    /// Extract ISO for inspection
    Extract {
        /// ISO file to extract
        iso: String,
        
        /// Output directory
        #[arg(short, long, default_value = "./iso-extract")]
        output: String,
    },
    /// Verify ISO integrity
    Verify {
        /// ISO file to verify
        iso: String,
    },
    /// Clean ISO build artifacts
    Clean,
}

#[derive(Subcommand)]
enum ReleaseAction {
    /// Create a new release
    Create {
        /// Version for the release
        version: Option<String>,
        
        /// Skip tests
        #[arg(long)]
        skip_tests: bool,
        
        /// Skip changelog generation
        #[arg(long)]
        skip_changelog: bool,
    },
    /// Generate changelog
    Changelog {
        /// Version range (e.g., v0.1.0..HEAD)
        #[arg(short, long)]
        range: Option<String>,
        
        /// Output format (markdown, json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
    /// Prepare release notes
    Notes {
        /// Version to generate notes for
        version: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    match cli.command {
        Commands::Version { action } => {
            handle_version_command(action).await?;
        }
        Commands::Commit { action } => {
            handle_commit_command(action).await?;
        }
        Commands::Check { only, fix } => {
            check::run_checks(only.as_deref(), fix).await?;
        }
        Commands::Release { action } => {
            handle_release_command(action).await?;
        }
        Commands::InitHooks { force } => {
            init_git_hooks(force).await?;
        }
        Commands::Build { action } => {
            handle_build_command(action).await?;
        }
        Commands::Iso { action } => {
            handle_iso_command(action).await?;
        }
        Commands::Doctor { fix } => {
            run_doctor(fix).await?;
        }
        Commands::Setup { auto } => {
            setup::setup_environment(auto).await?;
        }
    }

    Ok(())
}

async fn handle_version_command(action: VersionAction) -> Result<()> {
    match action {
        VersionAction::Show => {
            version::show_version()?;
        }
        VersionAction::Bump { level, dry_run } => {
            version::bump_version(level, dry_run)?;
        }
        VersionAction::Sync { version } => {
            version::sync_version(version.as_deref())?;
        }
        VersionAction::Check => {
            version::check_version_sync()?;
        }
    }
    Ok(())
}

async fn handle_commit_command(action: CommitAction) -> Result<()> {
    match action {
        CommitAction::Validate { message } => {
            commit::validate_commit(message.as_deref())?;
        }
        CommitAction::Create { 
            commit_type, 
            scope, 
            message, 
            breaking 
        } => {
            commit::create_commit(&commit_type, scope.as_deref(), &message, breaking)?;
        }
        CommitAction::Conventions => {
            commit::show_conventions();
        }
    }
    Ok(())
}

async fn handle_release_command(action: ReleaseAction) -> Result<()> {
    match action {
        ReleaseAction::Create { 
            version, 
            skip_tests, 
            skip_changelog 
        } => {
            release::create_release(
                version.as_deref(), 
                skip_tests, 
                skip_changelog
            ).await?;
        }
        ReleaseAction::Changelog { range, format } => {
            release::generate_changelog(range.as_deref(), &format)?;
        }
        ReleaseAction::Notes { version } => {
            release::generate_release_notes(version.as_deref())?;
        }
    }
    Ok(())
}

async fn handle_build_command(action: BuildAction) -> Result<()> {
    match action {
        BuildAction::All { release, test } => {
            build::build_all(release, test).await?;
        }
        BuildAction::Component { name, release } => {
            build::build_component(&name, release).await?;
        }
        BuildAction::Clean { deep } => {
            build::clean(deep).await?;
        }
        BuildAction::Status => {
            build::show_status().await?;
        }
    }
    Ok(())
}

async fn handle_iso_command(action: IsoAction) -> Result<()> {
    match action {
        IsoAction::Create { 
            download, 
            input, 
            output, 
            skip_build,
            with_source 
        } => {
            iso::create_iso(
                download.as_deref(),
                input.as_deref(),
                &output,
                skip_build,
                with_source
            ).await?;
        }
        IsoAction::Extract { iso, output } => {
            iso::extract_iso(&iso, &output).await?;
        }
        IsoAction::Verify { iso } => {
            iso::verify_iso(&iso).await?;
        }
        IsoAction::Clean => {
            iso::clean_artifacts().await?;
        }
    }
    Ok(())
}

async fn run_doctor(fix: bool) -> Result<()> {
    use crate::utils::*;
    
    print_header("HecateOS Doctor - System Check");
    
    // Check dependencies
    info_msg("Checking system dependencies...");
    let deps = check_dependencies()?;
    print_dependency_report(&deps);
    
    // Check project structure
    println!("\n{}", "Project Status".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    
    // Check if in correct directory
    let rust_dir = build::find_project_root();
    let has_project = rust_dir.is_ok();
    
    match rust_dir {
        Ok(dir) => {
            success_msg(&format!("Project found at: {}", dir.display()));
        }
        Err(e) => {
            error_msg(&format!("Project not found: {}", e));
            if fix {
                info_msg("Set HECATE_ROOT environment variable to the rust directory");
            }
        }
    }
    
    // Check build status
    if has_project {
        let status = build::show_status().await;
        if status.is_err() {
            warn_msg("Could not check build status");
        }
    }
    
    // Check for ISO tools
    println!("\n{}", "ISO Creation Capabilities".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    
    // Native ISO creation
    success_msg("✅ Native ISO creation: Built-in Rust implementation");
    info_msg("   hecate-iso can create ISOs without external tools!");
    
    // Check extraction tools (optional)
    if has_iso_extraction_tool() {
        success_msg("ISO extraction: External tool available (7z)");
    } else {
        info_msg("ISO extraction: Optional tool not installed");
        if fix {
            info_msg("   For faster extraction: sudo apt-get install p7zip-full");
            info_msg("   Note: Native extraction coming soon!");
        }
    }
    
    // Check for common issues
    println!("\n{}", "Common Issues".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    
    // Check if running as root
    if std::env::var("USER").unwrap_or_default() == "root" {
        warn_msg("Running as root is not recommended");
    }
    
    // Check available disk space
    if let Ok(output) = std::process::Command::new("df")
        .arg("-h")
        .arg(".")
        .output() 
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(percent) = parts[4].trim_end_matches('%').parse::<u32>() {
                    if percent > 90 {
                        error_msg(&format!("Low disk space: {}% used", percent));
                    } else if percent > 80 {
                        warn_msg(&format!("Disk space: {}% used", percent));
                    } else {
                        success_msg(&format!("Disk space: {}% used", percent));
                    }
                    break;
                }
            }
        }
    }
    
    // Final recommendations
    println!("\n{}", "Recommendations".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    
    if !has_iso_extraction_tool() {
        println!("Optional: For faster ISO extraction:");
        println!("   {}", "sudo apt-get install p7zip-full".bright_yellow());
    }
    
    println!("\n💡 {}", "ISO Creation".bright_green().bold());
    println!("   HecateOS includes native ISO creation in Rust!");
    println!("   No external dependencies required for creating ISOs.");
    println!("   Use: {}", "hecate-iso build".bright_cyan());
    
    if deps.get("git").map_or(true, |s| !s.installed) {
        println!("2. Install git:");
        println!("   {}", "sudo apt-get install git".bright_yellow());
    }
    
    if deps.get("cargo").map_or(true, |s| !s.installed) {
        println!("3. Install Rust:");
        println!("   {}", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".bright_yellow());
    }
    
    println!("\n{}", "Run 'hecate-dev doctor --fix' to see fix instructions".bright_black());
    
    Ok(())
}

async fn init_git_hooks(force: bool) -> Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    
    info!("Installing git hooks...");
    
    let hooks_dir = ".git/hooks";
    fs::create_dir_all(hooks_dir)?;
    
    // Pre-commit hook
    let pre_commit_path = format!("{}/pre-commit", hooks_dir);
    if !force && std::path::Path::new(&pre_commit_path).exists() {
        warn!("pre-commit hook already exists. Use --force to overwrite.");
    } else {
        let pre_commit_content = r#"#!/bin/sh
# HecateOS pre-commit hook

# Run hecate-dev checks
hecate-dev check --only structure,imports,licenses

# Validate commit message format
if [ -f .git/COMMIT_EDITMSG ]; then
    hecate-dev commit validate
fi

# Run tests
cargo test --quiet
"#;
        fs::write(&pre_commit_path, pre_commit_content)?;
        let mut perms = fs::metadata(&pre_commit_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms)?;
        info!("Installed pre-commit hook");
    }
    
    // Pre-push hook
    let pre_push_path = format!("{}/pre-push", hooks_dir);
    if !force && std::path::Path::new(&pre_push_path).exists() {
        warn!("pre-push hook already exists. Use --force to overwrite.");
    } else {
        let pre_push_content = r#"#!/bin/sh
# HecateOS pre-push hook

# Check version sync
hecate-dev version check

# Run full test suite
cargo test

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Uncommitted changes detected"
    exit 1
fi
"#;
        fs::write(&pre_push_path, pre_push_content)?;
        let mut perms = fs::metadata(&pre_push_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_push_path, perms)?;
        info!("Installed pre-push hook");
    }
    
    info!("Git hooks installed successfully");
    Ok(())
}