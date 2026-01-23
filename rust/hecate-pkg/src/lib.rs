//! HecateOS Package Manager Core Library
//! 
//! Modern, secure, and efficient package management for HecateOS

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use semver::Version;
use chrono::{DateTime, Utc};

// ============================================================================
// PACKAGE TYPES AND METADATA
// ============================================================================

/// Package metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub replaces: Vec<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub architecture: Architecture,
    pub size_bytes: u64,
    pub installed_size_bytes: u64,
    pub checksum: PackageChecksum,
    pub signature: Option<String>,
    pub build_date: DateTime<Utc>,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: String,  // Semver requirement string
    pub optional: bool,
    pub build_only: bool,
}

/// Package checksum for integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChecksum {
    pub sha256: String,
    pub blake3: String,
}

/// Supported architectures
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Riscv64,
    All,  // Architecture-independent packages
}

/// Package installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub package: Package,
    pub install_date: DateTime<Utc>,
    pub install_path: PathBuf,
    pub files: Vec<InstalledFile>,
    pub install_reason: InstallReason,
}

/// File installed by a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledFile {
    pub path: PathBuf,
    pub checksum: String,
    pub size: u64,
    pub permissions: u32,
}

/// Reason for package installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallReason {
    Explicit,      // User requested
    Dependency,    // Pulled in as dependency
    Group,         // Part of a group install
}

// ============================================================================
// REPOSITORY MANAGEMENT
// ============================================================================

/// Package repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub mirror_urls: Vec<String>,
    pub enabled: bool,
    pub priority: i32,  // Lower = higher priority
    pub gpg_check: bool,
    pub gpg_key: Option<String>,
    pub last_update: Option<DateTime<Utc>>,
}

/// Repository index containing package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub repository: Repository,
    pub packages: HashMap<String, Vec<Package>>,  // name -> versions
    pub groups: HashMap<String, Vec<String>>,     // group -> packages
    pub provides_index: HashMap<String, Vec<String>>,  // provides -> packages
}

// ============================================================================
// PACKAGE MANAGER CORE
// ============================================================================

/// Main package manager struct
pub struct PackageManager {
    config: PackageConfig,
    database: PackageDatabase,
    cache: PackageCache,
    repositories: Vec<Repository>,
}

/// Package manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub root_dir: PathBuf,
    pub db_path: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub parallel_downloads: usize,
    pub keep_cache: bool,
    pub verify_signatures: bool,
    pub auto_remove_orphans: bool,
    pub color_output: bool,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("/"),
            db_path: PathBuf::from("/var/lib/hecate-pkg/db"),
            cache_dir: PathBuf::from("/var/cache/hecate-pkg"),
            log_dir: PathBuf::from("/var/log/hecate-pkg"),
            parallel_downloads: 4,
            keep_cache: true,
            verify_signatures: true,
            auto_remove_orphans: false,
            color_output: true,
        }
    }
}

impl PackageManager {
    /// Create a new package manager instance
    pub async fn new(config: PackageConfig) -> Result<Self> {
        let database = PackageDatabase::open(&config.db_path).await?;
        let cache = PackageCache::new(&config.cache_dir)?;
        let repositories = Self::load_repositories(&config).await?;

        Ok(Self {
            config,
            database,
            cache,
            repositories,
        })
    }

    /// Load repository configurations
    async fn load_repositories(config: &PackageConfig) -> Result<Vec<Repository>> {
        let repos_dir = config.root_dir.join("etc/hecate-pkg/repos.d");
        let mut repositories = Vec::new();

        if repos_dir.exists() {
            for entry in std::fs::read_dir(&repos_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension() == Some(std::ffi::OsStr::new("repo")) {
                    let content = std::fs::read_to_string(&path)?;
                    let repo: Repository = toml::from_str(&content)?;
                    repositories.push(repo);
                }
            }
        }

        // Sort by priority
        repositories.sort_by_key(|r| r.priority);

        Ok(repositories)
    }

    /// Search for packages
    pub async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let mut results = Vec::new();

        for repo_index in self.database.get_repository_indices().await? {
            for (name, versions) in &repo_index.packages {
                if name.contains(query) {
                    results.extend(versions.clone());
                } else {
                    for pkg in versions {
                        if pkg.description.to_lowercase().contains(&query.to_lowercase()) 
                            || pkg.keywords.iter().any(|k| k.contains(query)) {
                            results.push(pkg.clone());
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Install a package
    pub async fn install(&mut self, package_name: &str) -> Result<()> {
        // Check if already installed
        if self.database.is_installed(package_name).await? {
            return Err(anyhow::anyhow!("Package {} is already installed", package_name));
        }

        // Find package in repositories
        let package = self.find_package(package_name).await?
            .ok_or_else(|| anyhow::anyhow!("Package {} not found", package_name))?;

        // Resolve dependencies
        let install_plan = self.resolve_dependencies(&package).await?;

        // Download packages
        for pkg in &install_plan {
            self.download_package(pkg).await?;
        }

        // Verify checksums
        for pkg in &install_plan {
            self.verify_package(pkg).await?;
        }

        // Install packages in order
        for pkg in install_plan {
            self.install_package(pkg).await?;
        }

        Ok(())
    }

    /// Remove a package
    pub async fn remove(&mut self, package_name: &str) -> Result<()> {
        // Check if installed
        if !self.database.is_installed(package_name).await? {
            return Err(anyhow::anyhow!("Package {} is not installed", package_name));
        }

        // Check for dependent packages
        let dependents = self.database.get_dependents(package_name).await?;
        if !dependents.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot remove {}: required by {:?}", 
                package_name, dependents
            ));
        }

        // Get installed package info
        let installed = self.database.get_installed_package(package_name).await?;

        // Remove files
        for file in installed.files.iter().rev() {
            if file.path.exists() {
                if file.path.is_dir() {
                    std::fs::remove_dir(&file.path)?;
                } else {
                    std::fs::remove_file(&file.path)?;
                }
            }
        }

        // Update database
        self.database.mark_removed(package_name).await?;

        // Remove orphaned dependencies if configured
        if self.config.auto_remove_orphans {
            self.remove_orphans().await?;
        }

        Ok(())
    }

    /// Update all packages
    pub async fn update(&mut self) -> Result<()> {
        // Update repository indices
        self.sync_repositories().await?;

        // Get list of installed packages
        let installed = self.database.get_installed_packages().await?;

        // Find updates
        let mut updates = Vec::new();
        for pkg in installed {
            if let Some(latest) = self.find_package(&pkg.package.name).await? {
                if latest.version > pkg.package.version {
                    updates.push((pkg.package.name.clone(), latest));
                }
            }
        }

        if updates.is_empty() {
            println!("All packages are up to date");
            return Ok(());
        }

        // Apply updates
        println!("Found {} updates", updates.len());
        for (name, pkg) in updates {
            println!("Updating {} from {} to {}", name, 
                self.database.get_installed_package(&name).await?.package.version,
                pkg.version
            );
            self.upgrade_package(pkg).await?;
        }

        Ok(())
    }

    /// Sync repository indices
    pub async fn sync_repositories(&mut self) -> Result<()> {
        use futures::stream::{self, StreamExt};

        let repos = self.repositories.clone();
        let tasks = repos.into_iter()
            .filter(|r| r.enabled)
            .map(|repo| self.sync_repository(repo));

        let results: Vec<Result<()>> = stream::iter(tasks)
            .buffer_unordered(self.config.parallel_downloads)
            .collect()
            .await;

        for result in results {
            result?;
        }

        Ok(())
    }

    /// Sync a single repository
    async fn sync_repository(&self, repo: Repository) -> Result<()> {
        let index_url = format!("{}/index.json.zst", repo.url);
        
        // Download compressed index
        let response = reqwest::get(&index_url).await?;
        let compressed_data = response.bytes().await?;

        // Decompress
        let data = zstd::decode_all(compressed_data.as_ref())?;

        // Parse index
        let index: RepositoryIndex = serde_json::from_slice(&data)?;

        // Verify signature if enabled
        if repo.gpg_check {
            // TODO: Implement GPG verification
        }

        // Save to database
        self.database.update_repository_index(index).await?;

        Ok(())
    }

    /// Find a package in repositories
    async fn find_package(&self, name: &str) -> Result<Option<Package>> {
        for repo_index in self.database.get_repository_indices().await? {
            if let Some(versions) = repo_index.packages.get(name) {
                // Return latest version
                if let Some(latest) = versions.iter().max_by_key(|p| &p.version) {
                    return Ok(Some(latest.clone()));
                }
            }
        }
        Ok(None)
    }

    /// Resolve package dependencies
    async fn resolve_dependencies(&self, package: &Package) -> Result<Vec<Package>> {
        let mut to_install = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.resolve_deps_recursive(package, &mut to_install, &mut visited).await?;

        // Reverse to get correct installation order
        to_install.reverse();
        Ok(to_install)
    }

    async fn resolve_deps_recursive(
        &self,
        package: &Package,
        to_install: &mut Vec<Package>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if visited.contains(&package.name) {
            return Ok(());
        }
        visited.insert(package.name.clone());

        for dep in &package.dependencies {
            if dep.optional || dep.build_only {
                continue;
            }

            // Skip if already installed and satisfies requirement
            if self.database.is_installed(&dep.name).await? {
                let installed = self.database.get_installed_package(&dep.name).await?;
                let req = semver::VersionReq::parse(&dep.version_req)?;
                if req.matches(&installed.package.version) {
                    continue;
                }
            }

            // Find dependency package
            if let Some(dep_pkg) = self.find_package(&dep.name).await? {
                self.resolve_deps_recursive(&dep_pkg, to_install, visited).await?;
            } else {
                return Err(anyhow::anyhow!("Dependency {} not found", dep.name));
            }
        }

        to_install.push(package.clone());
        Ok(())
    }

    /// Download a package
    async fn download_package(&self, package: &Package) -> Result<PathBuf> {
        let cache_path = self.cache.get_package_path(package);
        
        if cache_path.exists() {
            // Verify cached package
            if self.verify_cached_package(package, &cache_path).await? {
                return Ok(cache_path);
            }
        }

        // Find download URL
        let download_url = self.get_package_url(package).await?;

        // Download with progress
        let response = reqwest::get(&download_url).await?;
        let total_size = response.content_length().unwrap_or(package.size_bytes);

        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}")?
                .progress_chars("##-"),
        );
        pb.set_message(format!("Downloading {}", package.name));

        // Stream to file
        let mut file = tokio::fs::File::create(&cache_path).await?;
        let mut stream = response.bytes_stream();

        use tokio::io::AsyncWriteExt;
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message(format!("Downloaded {}", package.name));

        Ok(cache_path)
    }

    /// Verify package integrity
    async fn verify_package(&self, package: &Package) -> Result<()> {
        let cache_path = self.cache.get_package_path(package);
        
        // Calculate checksums
        let data = tokio::fs::read(&cache_path).await?;
        
        use sha2::{Sha256, Digest};
        let sha256 = hex::encode(Sha256::digest(&data));
        let blake3 = hex::encode(blake3::hash(&data).as_bytes());

        // Verify checksums
        if sha256 != package.checksum.sha256 {
            return Err(anyhow::anyhow!("SHA256 checksum mismatch for {}", package.name));
        }

        if blake3 != package.checksum.blake3 {
            return Err(anyhow::anyhow!("BLAKE3 checksum mismatch for {}", package.name));
        }

        // Verify signature if present
        if self.config.verify_signatures {
            if let Some(ref signature) = package.signature {
                // TODO: Implement signature verification
            }
        }

        Ok(())
    }

    /// Install a package from cache
    async fn install_package(&mut self, package: Package) -> Result<()> {
        let cache_path = self.cache.get_package_path(&package);
        let install_root = &self.config.root_dir;

        // Extract package
        let tar = std::fs::File::open(&cache_path)?;
        let decoder = zstd::Decoder::new(tar)?;
        let mut archive = tar::Archive::new(decoder);

        let mut installed_files = Vec::new();

        // Track installed files
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let install_path = install_root.join(&path);

            // Create parent directories
            if let Some(parent) = install_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Extract file
            entry.unpack(&install_path)?;

            // Record installed file
            let metadata = install_path.metadata()?;
            installed_files.push(InstalledFile {
                path: path.to_path_buf(),
                checksum: String::new(),  // TODO: Calculate file checksum
                size: metadata.len(),
                permissions: 0o644,  // TODO: Get actual permissions
            });
        }

        // Record installation in database
        let installed = InstalledPackage {
            package,
            install_date: Utc::now(),
            install_path: install_root.to_path_buf(),
            files: installed_files,
            install_reason: InstallReason::Explicit,
        };

        self.database.record_installation(installed).await?;

        Ok(())
    }

    /// Upgrade a package
    async fn upgrade_package(&mut self, package: Package) -> Result<()> {
        let old_version = self.database.get_installed_package(&package.name).await?;
        
        // Download new version
        self.download_package(&package).await?;
        
        // Verify new package
        self.verify_package(&package).await?;
        
        // Backup configuration files
        let config_files = self.backup_config_files(&old_version).await?;
        
        // Remove old version
        self.remove(&package.name).await?;
        
        // Install new version
        self.install_package(package).await?;
        
        // Restore configuration files
        self.restore_config_files(config_files).await?;
        
        Ok(())
    }

    /// Remove orphaned packages
    async fn remove_orphans(&mut self) -> Result<()> {
        let orphans = self.database.find_orphans().await?;
        
        for orphan in orphans {
            println!("Removing orphaned package: {}", orphan);
            self.remove(&orphan).await?;
        }
        
        Ok(())
    }

    /// Verify cached package
    async fn verify_cached_package(&self, package: &Package, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let data = tokio::fs::read(path).await?;
        
        use sha2::{Sha256, Digest};
        let sha256 = hex::encode(Sha256::digest(&data));
        
        Ok(sha256 == package.checksum.sha256)
    }

    /// Get package download URL
    async fn get_package_url(&self, package: &Package) -> Result<String> {
        // Find repository containing this package
        for repo in &self.repositories {
            // Check if repository has this package
            // TODO: Implement proper URL construction
            let url = format!("{}/packages/{}-{}.pkg.tar.zst", 
                repo.url, package.name, package.version);
            return Ok(url);
        }
        
        Err(anyhow::anyhow!("No repository contains package {}", package.name))
    }

    /// Backup configuration files
    async fn backup_config_files(&self, installed: &InstalledPackage) -> Result<Vec<PathBuf>> {
        let mut config_files = Vec::new();
        
        for file in &installed.files {
            if file.path.starts_with("/etc") {
                let backup_path = file.path.with_extension("hecate-backup");
                tokio::fs::copy(&file.path, &backup_path).await?;
                config_files.push(backup_path);
            }
        }
        
        Ok(config_files)
    }

    /// Restore configuration files
    async fn restore_config_files(&self, backups: Vec<PathBuf>) -> Result<()> {
        for backup in backups {
            if backup.exists() {
                let original = backup.with_extension("");
                
                // Check if new config differs from old
                let old_content = tokio::fs::read(&backup).await?;
                let new_content = tokio::fs::read(&original).await?;
                
                if old_content != new_content {
                    // Keep both versions
                    let new_path = original.with_extension("hecate-new");
                    tokio::fs::rename(&original, &new_path).await?;
                    tokio::fs::rename(&backup, &original).await?;
                    
                    println!("Configuration file {} has been modified.", original.display());
                    println!("  Old version: {}", original.display());
                    println!("  New version: {}", new_path.display());
                } else {
                    // Remove backup
                    tokio::fs::remove_file(&backup).await?;
                }
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// DATABASE
// ============================================================================

/// Package database for tracking installations
struct PackageDatabase {
    pool: sqlx::SqlitePool,
}

impl PackageDatabase {
    async fn open(path: &Path) -> Result<Self> {
        // Create database directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let database_url = format!("sqlite://{}", path.display());
        let pool = sqlx::SqlitePool::connect(&database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    async fn is_installed(&self, package_name: &str) -> Result<bool> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM installed_packages WHERE name = ?",
            package_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count > 0)
    }

    async fn get_installed_package(&self, name: &str) -> Result<InstalledPackage> {
        // TODO: Implement database query
        unimplemented!()
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        // TODO: Implement database query
        unimplemented!()
    }

    async fn get_dependents(&self, package_name: &str) -> Result<Vec<String>> {
        // TODO: Implement database query
        unimplemented!()
    }

    async fn mark_removed(&self, package_name: &str) -> Result<()> {
        // TODO: Implement database update
        unimplemented!()
    }

    async fn record_installation(&self, installed: InstalledPackage) -> Result<()> {
        // TODO: Implement database insert
        unimplemented!()
    }

    async fn find_orphans(&self) -> Result<Vec<String>> {
        // TODO: Implement orphan detection
        unimplemented!()
    }

    async fn get_repository_indices(&self) -> Result<Vec<RepositoryIndex>> {
        // TODO: Implement repository index retrieval
        unimplemented!()
    }

    async fn update_repository_index(&self, index: RepositoryIndex) -> Result<()> {
        // TODO: Implement repository index update
        unimplemented!()
    }
}

// ============================================================================
// CACHE
// ============================================================================

/// Package cache for downloaded packages
struct PackageCache {
    cache_dir: PathBuf,
}

impl PackageCache {
    fn new(cache_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(cache_dir)?;
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
        })
    }

    fn get_package_path(&self, package: &Package) -> PathBuf {
        let filename = format!("{}-{}.pkg.tar.zst", package.name, package.version);
        self.cache_dir.join(filename)
    }

    fn clean(&self) -> Result<()> {
        // Remove old cached packages
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Check age of file
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified.elapsed().unwrap_or_default().as_secs() > 30 * 24 * 3600 {
                        std::fs::remove_file(&path)?;
                    }
                }
            }
        }
        Ok(())
    }
}