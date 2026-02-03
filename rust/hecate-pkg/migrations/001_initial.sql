-- Initial database schema for hecate-pkg

-- Repository information
CREATE TABLE IF NOT EXISTS repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 50,
    gpg_check BOOLEAN NOT NULL DEFAULT 1,
    gpg_key TEXT,
    last_update TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Available packages from repositories
CREATE TABLE IF NOT EXISTS available_packages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repository_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT,
    license TEXT,
    homepage TEXT,
    repository_url TEXT,
    architecture TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    installed_size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    blake3 TEXT NOT NULL,
    signature TEXT,
    build_date TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
    UNIQUE(repository_id, name, version)
);

-- Installed packages
CREATE TABLE IF NOT EXISTS installed_packages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT,
    license TEXT,
    architecture TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    install_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    install_path TEXT NOT NULL,
    install_reason TEXT NOT NULL CHECK(install_reason IN ('explicit', 'dependency', 'group')),
    sha256 TEXT NOT NULL,
    blake3 TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Files installed by packages
CREATE TABLE IF NOT EXISTS installed_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    checksum TEXT,
    size INTEGER NOT NULL,
    permissions INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (package_id) REFERENCES installed_packages(id) ON DELETE CASCADE
);

-- Package dependencies
CREATE TABLE IF NOT EXISTS dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL,
    depends_on TEXT NOT NULL,
    version_req TEXT,
    optional BOOLEAN NOT NULL DEFAULT 0,
    build_only BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (package_id) REFERENCES installed_packages(id) ON DELETE CASCADE
);

-- Package provides (virtual packages)
CREATE TABLE IF NOT EXISTS provides (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL,
    provides TEXT NOT NULL,
    FOREIGN KEY (package_id) REFERENCES installed_packages(id) ON DELETE CASCADE
);

-- Package conflicts
CREATE TABLE IF NOT EXISTS conflicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL,
    conflicts_with TEXT NOT NULL,
    FOREIGN KEY (package_id) REFERENCES installed_packages(id) ON DELETE CASCADE
);

-- Package groups
CREATE TABLE IF NOT EXISTS package_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Group members
CREATE TABLE IF NOT EXISTS group_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    package_name TEXT NOT NULL,
    FOREIGN KEY (group_id) REFERENCES package_groups(id) ON DELETE CASCADE,
    UNIQUE(group_id, package_name)
);

-- Repository index cache
CREATE TABLE IF NOT EXISTS repository_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repository_id INTEGER NOT NULL,
    data BLOB NOT NULL,  -- Compressed JSON data
    checksum TEXT NOT NULL,
    last_update TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
);

-- Transaction log for rollback support
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('install', 'remove', 'upgrade')),
    package_name TEXT NOT NULL,
    old_version TEXT,
    new_version TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'completed', 'failed', 'rolled_back')),
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_available_packages_name ON available_packages(name);
CREATE INDEX IF NOT EXISTS idx_available_packages_repo ON available_packages(repository_id);
CREATE INDEX IF NOT EXISTS idx_installed_packages_name ON installed_packages(name);
CREATE INDEX IF NOT EXISTS idx_installed_files_package ON installed_files(package_id);
CREATE INDEX IF NOT EXISTS idx_installed_files_path ON installed_files(path);
CREATE INDEX IF NOT EXISTS idx_dependencies_package ON dependencies(package_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_depends ON dependencies(depends_on);
CREATE INDEX IF NOT EXISTS idx_provides_package ON provides(package_id);
CREATE INDEX IF NOT EXISTS idx_provides_name ON provides(provides);
CREATE INDEX IF NOT EXISTS idx_conflicts_package ON conflicts(package_id);
CREATE INDEX IF NOT EXISTS idx_group_members_group ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);

-- Triggers for updated_at
CREATE TRIGGER IF NOT EXISTS update_repositories_timestamp 
AFTER UPDATE ON repositories 
BEGIN
    UPDATE repositories SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_installed_packages_timestamp 
AFTER UPDATE ON installed_packages 
BEGIN
    UPDATE installed_packages SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;