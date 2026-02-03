//! Database operations for package management
//!
//! Provides SQLite-based storage for package metadata, installation tracking,
//! and dependency resolution.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;
use chrono::{DateTime, Utc};

use crate::{
    Package, InstalledPackage, InstalledFile, InstallReason,
    RepositoryIndex, Repository, Architecture, PackageChecksum,
    Dependency,
};

/// Package database for tracking installations
pub struct PackageDatabase {
    pool: SqlitePool,
}

impl PackageDatabase {
    /// Open or create a package database
    pub async fn open(path: &Path) -> Result<Self> {
        // Create database directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        // Connect to database
        let database_url = format!("sqlite://{}", path.display());
        let pool = SqlitePool::connect(&database_url)
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    /// Run database migrations
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // Read migration SQL
        let migration = include_str!("../migrations/001_initial.sql");
        
        // Execute migration
        sqlx::query(migration)
            .execute(pool)
            .await
            .context("Failed to run database migrations")?;

        Ok(())
    }

    /// Check if a package is installed
    pub async fn is_installed(&self, package_name: &str) -> Result<bool> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM installed_packages WHERE name = ?")
            .bind(package_name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0 > 0)
    }

    /// Get installed package information
    pub async fn get_installed_package(&self, name: &str) -> Result<InstalledPackage> {
        // Fetch package data
        let row: (i64, String, String, Option<String>, Option<String>, Option<String>,
                 String, i64, String, String, String, String, String) = sqlx::query_as(
            r#"
            SELECT id, name, version, description, author, license,
                   architecture, size_bytes, install_date, install_path,
                   install_reason, sha256, blake3
            FROM installed_packages
            WHERE name = ?
            "#
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .context("Package not found")?;

        // Fetch installed files
        let files: Vec<(String, Option<String>, i64, i64)> = sqlx::query_as(
            r#"
            SELECT path, checksum, size, permissions
            FROM installed_files
            WHERE package_id = ?
            ORDER BY path
            "#
        )
        .bind(row.0)
        .fetch_all(&self.pool)
        .await?;

        // Fetch dependencies
        let deps: Vec<(String, Option<String>, i32, i32)> = sqlx::query_as(
            r#"
            SELECT depends_on, version_req, optional, build_only
            FROM dependencies
            WHERE package_id = ?
            "#
        )
        .bind(row.0)
        .fetch_all(&self.pool)
        .await?;

        // Convert to InstalledPackage
        let package = Package {
            name: row.1,
            version: semver::Version::parse(&row.2)?,
            description: row.3.unwrap_or_default(),
            author: row.4.unwrap_or_default(),
            license: row.5.unwrap_or_default(),
            homepage: None,
            repository: None,
            dependencies: deps.into_iter().map(|d| Dependency {
                name: d.0,
                version_req: d.1.unwrap_or_default(),
                optional: d.2 != 0,
                build_only: d.3 != 0,
            }).collect(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            categories: Vec::new(),
            keywords: Vec::new(),
            architecture: match row.6.as_str() {
                "x86_64" => Architecture::X86_64,
                "aarch64" => Architecture::Aarch64,
                "riscv64" => Architecture::Riscv64,
                _ => Architecture::All,
            },
            size_bytes: row.7 as u64,
            installed_size_bytes: row.7 as u64,
            checksum: PackageChecksum {
                sha256: row.11,
                blake3: row.12,
            },
            signature: None,
            build_date: Utc::now(),
        };

        let installed_files = files.into_iter().map(|f| InstalledFile {
            path: f.0.into(),
            checksum: f.1.unwrap_or_default(),
            size: f.2 as u64,
            permissions: f.3 as u32,
        }).collect();

        let install_reason = match row.10.as_str() {
            "dependency" => InstallReason::Dependency,
            "group" => InstallReason::Group,
            _ => InstallReason::Explicit,
        };

        Ok(InstalledPackage {
            package,
            install_date: DateTime::parse_from_rfc3339(&row.8)?.with_timezone(&Utc),
            install_path: row.9.into(),
            files: installed_files,
            install_reason,
        })
    }

    /// Get all installed packages
    pub async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        let names: Vec<(String,)> = sqlx::query_as("SELECT name FROM installed_packages")
            .fetch_all(&self.pool)
            .await?;

        let mut packages = Vec::new();
        for row in names {
            if let Ok(pkg) = self.get_installed_package(&row.0).await {
                packages.push(pkg);
            }
        }

        Ok(packages)
    }

    /// Get packages that depend on a specific package
    pub async fn get_dependents(&self, package_name: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ip.name
            FROM installed_packages ip
            JOIN dependencies d ON ip.id = d.package_id
            WHERE d.depends_on = ?
            "#
        )
        .bind(package_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Mark a package as removed
    pub async fn mark_removed(&self, package_name: &str) -> Result<()> {
        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Get package ID
        let row: (i64,) = sqlx::query_as("SELECT id FROM installed_packages WHERE name = ?")
            .bind(package_name)
            .fetch_one(&mut *tx)
            .await?;

        // Delete installed files
        sqlx::query("DELETE FROM installed_files WHERE package_id = ?")
            .bind(row.0)
            .execute(&mut *tx)
            .await?;

        // Delete dependencies
        sqlx::query("DELETE FROM dependencies WHERE package_id = ?")
            .bind(row.0)
            .execute(&mut *tx)
            .await?;

        // Delete provides
        sqlx::query("DELETE FROM provides WHERE package_id = ?")
            .bind(row.0)
            .execute(&mut *tx)
            .await?;

        // Delete conflicts
        sqlx::query("DELETE FROM conflicts WHERE package_id = ?")
            .bind(row.0)
            .execute(&mut *tx)
            .await?;

        // Delete package
        sqlx::query("DELETE FROM installed_packages WHERE id = ?")
            .bind(row.0)
            .execute(&mut *tx)
            .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(())
    }

    /// Record a package installation
    pub async fn record_installation(&self, installed: InstalledPackage) -> Result<()> {
        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Insert package
        let install_reason = match installed.install_reason {
            InstallReason::Explicit => "explicit",
            InstallReason::Dependency => "dependency",
            InstallReason::Group => "group",
        };

        let architecture = match installed.package.architecture {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
            Architecture::Riscv64 => "riscv64",
            Architecture::All => "all",
        };

        let package_id = sqlx::query(
            r#"
            INSERT INTO installed_packages 
            (name, version, description, author, license, architecture, 
             size_bytes, install_date, install_path, install_reason, sha256, blake3)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&installed.package.name)
        .bind(installed.package.version.to_string())
        .bind(&installed.package.description)
        .bind(&installed.package.author)
        .bind(&installed.package.license)
        .bind(architecture)
        .bind(installed.package.size_bytes as i64)
        .bind(installed.install_date.to_rfc3339())
        .bind(installed.install_path.to_string_lossy().as_ref())
        .bind(install_reason)
        .bind(&installed.package.checksum.sha256)
        .bind(&installed.package.checksum.blake3)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

        // Insert files
        for file in &installed.files {
            sqlx::query(
                r#"
                INSERT INTO installed_files (package_id, path, checksum, size, permissions)
                VALUES (?, ?, ?, ?, ?)
                "#
            )
            .bind(package_id)
            .bind(file.path.to_string_lossy().as_ref())
            .bind(&file.checksum)
            .bind(file.size as i64)
            .bind(file.permissions as i64)
            .execute(&mut *tx)
            .await?;
        }

        // Insert dependencies
        for dep in &installed.package.dependencies {
            sqlx::query(
                r#"
                INSERT INTO dependencies (package_id, depends_on, version_req, optional, build_only)
                VALUES (?, ?, ?, ?, ?)
                "#
            )
            .bind(package_id)
            .bind(&dep.name)
            .bind(&dep.version_req)
            .bind(dep.optional as i32)
            .bind(dep.build_only as i32)
            .execute(&mut *tx)
            .await?;
        }

        // Insert provides
        for provides in &installed.package.provides {
            sqlx::query(
                r#"
                INSERT INTO provides (package_id, provides)
                VALUES (?, ?)
                "#
            )
            .bind(package_id)
            .bind(provides)
            .execute(&mut *tx)
            .await?;
        }

        // Insert conflicts
        for conflicts in &installed.package.conflicts {
            sqlx::query(
                r#"
                INSERT INTO conflicts (package_id, conflicts_with)
                VALUES (?, ?)
                "#
            )
            .bind(package_id)
            .bind(conflicts)
            .execute(&mut *tx)
            .await?;
        }

        // Commit transaction
        tx.commit().await?;

        Ok(())
    }

    /// Find orphaned packages (installed as dependencies but no longer needed)
    pub async fn find_orphans(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT name FROM installed_packages
            WHERE install_reason = 'dependency'
            AND name NOT IN (
                SELECT DISTINCT depends_on FROM dependencies
                WHERE depends_on IS NOT NULL
            )
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Get all repository indices
    pub async fn get_repository_indices(&self) -> Result<Vec<RepositoryIndex>> {
        let rows: Vec<(i64, String, String, i32, i32, i32, Option<String>, Option<String>, Option<Vec<u8>>)> = sqlx::query_as(
            r#"
            SELECT r.id, r.name, r.url, r.enabled, r.priority, r.gpg_check,
                   r.gpg_key, r.last_update, ri.data
            FROM repositories r
            LEFT JOIN repository_index ri ON r.id = ri.repository_id
            WHERE r.enabled = 1
            ORDER BY r.priority
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut indices = Vec::new();
        
        for row in rows {
            if let Some(data) = row.8 {
                // Decompress and parse index data
                let decompressed = zstd::decode_all(data.as_slice())?;
                let mut index: RepositoryIndex = serde_json::from_slice(&decompressed)?;
                
                // Update repository info
                index.repository = Repository {
                    name: row.1,
                    url: row.2,
                    mirror_urls: Vec::new(),
                    enabled: row.3 != 0,
                    priority: row.4 as i32,
                    gpg_check: row.5 != 0,
                    gpg_key: row.6,
                    last_update: row.7.as_ref().and_then(|s| 
                        DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
                    ),
                };
                
                indices.push(index);
            }
        }

        Ok(indices)
    }

    /// Update repository index
    pub async fn update_repository_index(&self, index: RepositoryIndex) -> Result<()> {
        // Serialize and compress index
        let json = serde_json::to_vec(&index)?;
        let compressed = zstd::encode_all(json.as_slice(), 3)?;
        
        // Calculate checksum
        use sha2::{Sha256, Digest};
        let checksum = hex::encode(Sha256::digest(&compressed));
        
        // Start transaction
        let mut tx = self.pool.begin().await?;
        
        // Ensure repository exists
        let repo_id = sqlx::query(
            r#"
            INSERT OR REPLACE INTO repositories (name, url, enabled, priority, gpg_check)
            VALUES (?, ?, 1, ?, ?)
            "#
        )
        .bind(&index.repository.name)
        .bind(&index.repository.url)
        .bind(index.repository.priority)
        .bind(index.repository.gpg_check as i32)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        
        // Update repository index
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO repository_index (repository_id, data, checksum)
            VALUES (?, ?, ?)
            "#
        )
        .bind(repo_id)
        .bind(&compressed)
        .bind(&checksum)
        .execute(&mut *tx)
        .await?;
        
        // Update repository last_update
        sqlx::query("UPDATE repositories SET last_update = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(repo_id)
            .execute(&mut *tx)
            .await?;
        
        // Clear old available packages for this repository
        sqlx::query("DELETE FROM available_packages WHERE repository_id = ?")
            .bind(repo_id)
            .execute(&mut *tx)
            .await?;
        
        // Insert available packages
        for (_name, versions) in &index.packages {
            for pkg in versions {
                let architecture = match pkg.architecture {
                    Architecture::X86_64 => "x86_64",
                    Architecture::Aarch64 => "aarch64",
                    Architecture::Riscv64 => "riscv64",
                    Architecture::All => "all",
                };
                
                sqlx::query(
                    r#"
                    INSERT INTO available_packages
                    (repository_id, name, version, description, author, license,
                     homepage, repository_url, architecture, size_bytes, 
                     installed_size_bytes, sha256, blake3, signature, build_date)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(repo_id)
                .bind(&pkg.name)
                .bind(pkg.version.to_string())
                .bind(&pkg.description)
                .bind(&pkg.author)
                .bind(&pkg.license)
                .bind(pkg.homepage.as_ref())
                .bind(pkg.repository.as_ref())
                .bind(architecture)
                .bind(pkg.size_bytes as i64)
                .bind(pkg.installed_size_bytes as i64)
                .bind(&pkg.checksum.sha256)
                .bind(&pkg.checksum.blake3)
                .bind(pkg.signature.as_ref())
                .bind(pkg.build_date.to_rfc3339())
                .execute(&mut *tx)
                .await?;
            }
        }
        
        // Update package groups
        for (group_name, packages) in &index.groups {
            // Insert or get group ID
            let group_id = sqlx::query("INSERT OR IGNORE INTO package_groups (name) VALUES (?)")
                .bind(group_name)
                .execute(&mut *tx)
                .await?
                .last_insert_rowid();
            
            // Clear old members
            sqlx::query("DELETE FROM group_members WHERE group_id = ?")
                .bind(group_id)
                .execute(&mut *tx)
                .await?;
            
            // Insert new members
            for package in packages {
                sqlx::query("INSERT INTO group_members (group_id, package_name) VALUES (?, ?)")
                    .bind(group_id)
                    .bind(package)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        
        // Commit transaction
        tx.commit().await?;
        
        Ok(())
    }

    /// Begin a new transaction
    pub async fn begin_transaction(&self, transaction_type: &str, package_name: &str, old_version: Option<&str>, new_version: Option<&str>) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO transactions (transaction_type, package_name, old_version, new_version, status)
            VALUES (?, ?, ?, ?, 'pending')
            "#
        )
        .bind(transaction_type)
        .bind(package_name)
        .bind(old_version)
        .bind(new_version)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }

    /// Complete a transaction
    pub async fn complete_transaction(&self, transaction_id: i64) -> Result<()> {
        sqlx::query("UPDATE transactions SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(transaction_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    /// Fail a transaction
    pub async fn fail_transaction(&self, transaction_id: i64, error: &str) -> Result<()> {
        sqlx::query("UPDATE transactions SET status = 'failed', completed_at = CURRENT_TIMESTAMP, error_message = ? WHERE id = ?")
            .bind(error)
            .bind(transaction_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    /// Get package groups
    pub async fn get_groups(&self) -> Result<Vec<(String, String)>> {
        let rows: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT name, description FROM package_groups ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| (r.0, r.1.unwrap_or_default())).collect())
    }

    /// Get group members
    pub async fn get_group_members(&self, group_name: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT gm.package_name
            FROM group_members gm
            JOIN package_groups g ON gm.group_id = g.id
            WHERE g.name = ?
            ORDER BY gm.package_name
            "#
        )
        .bind(group_name)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Clean old cache entries
    pub async fn clean_cache(&self, keep_days: i32) -> Result<u64> {
        let cutoff = format!("datetime('now', '-{} days')", keep_days);
        
        let result = sqlx::query(&format!(
            "DELETE FROM repository_index WHERE last_update < {}",
            cutoff
        ))
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let installed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM installed_packages")
            .fetch_one(&self.pool)
            .await?;
        let installed = installed.0;
        
        let explicit: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM installed_packages WHERE install_reason = 'explicit'"
        )
        .fetch_one(&self.pool)
        .await?;
        let explicit = explicit.0;
        
        let available: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT name) FROM available_packages")
            .fetch_one(&self.pool)
            .await?;
        let available = available.0;
        
        let repositories: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repositories WHERE enabled = 1")
            .fetch_one(&self.pool)
            .await?;
        let repositories = repositories.0;
        
        let total_size: (Option<i64>,) = sqlx::query_as(
            "SELECT SUM(size_bytes) FROM installed_packages"
        )
        .fetch_one(&self.pool)
        .await?;
        let total_size = total_size.0.unwrap_or(0);
        
        let orphans = self.find_orphans().await?.len() as i64;
        
        Ok(DatabaseStats {
            installed_packages: installed,
            explicit_packages: explicit,
            dependency_packages: installed - explicit,
            orphaned_packages: orphans,
            available_packages: available,
            repositories,
            total_installed_size: total_size as u64,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub installed_packages: i64,
    pub explicit_packages: i64,
    pub dependency_packages: i64,
    pub orphaned_packages: i64,
    pub available_packages: i64,
    pub repositories: i64,
    pub total_installed_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let db = PackageDatabase::open(&db_path).await.unwrap();
        
        // Test basic operations
        assert!(!db.is_installed("test-package").await.unwrap());
        
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.installed_packages, 0);
    }
}