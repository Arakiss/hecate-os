# HecateOS Development Tools

A comprehensive suite of Rust-based development tools to ensure code quality, consistency, and proper development workflows for HecateOS.

## Installation

Build all development tools:

```bash
cd rust
cargo build --release --workspace
```

Install to system:

```bash
sudo cp target/release/hecate-* /usr/local/bin/
```

## Tools Overview

### 1. hecate-dev - Main Development CLI

Central command-line tool for all development workflows.

```bash
# Show current version
hecate-dev version show

# Bump version (major, minor, patch, prerelease)
hecate-dev version bump minor

# Sync versions across all files
hecate-dev version sync

# Check version consistency
hecate-dev version check

# Validate commit message
hecate-dev commit validate

# Create properly formatted commit
hecate-dev commit create -t feat -s rust -m "add GPU monitoring" --breaking

# Show commit conventions
hecate-dev commit conventions

# Run all checks
hecate-dev check

# Run specific checks with auto-fix
hecate-dev check --only structure,imports --fix

# Create a new release
hecate-dev release create --version 0.2.0

# Generate changelog
hecate-dev release changelog --range v0.1.0..HEAD

# Install git hooks
hecate-dev init-hooks --force
```

### 2. hecate-lint - Code Quality Enforcer

Static analysis and linting beyond standard Rust tools.

```bash
# Lint current directory
hecate-lint

# Lint specific path
hecate-lint rust/

# Auto-fix issues
hecate-lint --fix

# Check specific rules
hecate-lint --rules license-header,line-length
```

**Checks performed:**
- License headers in all source files
- TODO/FIXME comment tracking
- Line length limits (120 characters)
- File structure validation
- Import organization
- Configuration file validity

### 3. hecate-hooks - Git Hook Manager

Manages pre-commit, commit-msg, and pre-push hooks.

```bash
# Install all git hooks
hecate-hooks install

# Force reinstall hooks
hecate-hooks install --force

# Uninstall hooks
hecate-hooks uninstall

# Run pre-commit checks manually
hecate-hooks pre-commit

# Run pre-push checks manually
hecate-hooks pre-push

# Validate commit message
hecate-hooks commit-msg .git/COMMIT_EDITMSG
```

**Hooks installed:**
- **pre-commit**: Checks for merge conflicts, file sizes
- **commit-msg**: Validates conventional commit format
- **pre-push**: Runs tests, checks for uncommitted changes

### 4. hecate-changelog - Changelog Generator

Automatic changelog generation from conventional commits.

```bash
# Generate changelog for latest commits
hecate-changelog

# Generate for specific version
hecate-changelog --version 0.2.0

# Specify git range
hecate-changelog --range v0.1.0..HEAD

# Output formats
hecate-changelog --format json
hecate-changelog --format html

# Write to file
hecate-changelog --output changelog.md

# Update CHANGELOG.md directly
hecate-changelog --update --version 0.2.0
```

### 5. hecate-deps - Dependency Manager

Track and validate dependencies.

```bash
# Check for outdated dependencies
hecate-deps check

# Show dependency tree
hecate-deps tree

# Analyze licenses
hecate-deps licenses

# Security audit
hecate-deps audit

# Show binary size impact
hecate-deps size
```

### 6. hecate-arch - Architecture Validator

Ensure architectural consistency.

```bash
# Validate project structure
hecate-arch validate

# Check for circular dependencies
hecate-arch cycles

# Show module boundaries
hecate-arch boundaries

# Validate port configuration
hecate-arch ports

# Generate architecture diagram
hecate-arch diagram
```

## Conventional Commits

HecateOS follows the Conventional Commits specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature (→ minor version bump)
- **fix**: Bug fix (→ patch version bump)
- **docs**: Documentation only changes
- **style**: Formatting, white-space, etc.
- **refactor**: Code refactoring
- **perf**: Performance improvements
- **test**: Adding or correcting tests
- **chore**: Maintenance tasks
- **build**: Build system changes
- **ci**: CI configuration changes
- **revert**: Reverts a previous commit

### Scopes

- **rust**: Rust components
- **dashboard**: Web dashboard
- **iso**: ISO build system
- **docs**: Documentation
- **deps**: Dependencies

### Breaking Changes

Add `BREAKING CHANGE:` in the footer to indicate breaking changes (→ major version bump).

## Version Management

HecateOS uses Semantic Versioning (SemVer):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)
- **PRERELEASE**: Alpha, beta, rc versions

Version is synchronized across:
- `/VERSION` file
- `rust/Cargo.toml` (workspace version)
- All member `Cargo.toml` files
- `hecate-dashboard/package.json`

## CI/CD Integration

### GitHub Actions

```yaml
name: Development Checks

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build dev tools
        run: |
          cd rust
          cargo build --release
      
      - name: Run checks
        run: |
          ./target/release/hecate-dev check
          ./target/release/hecate-lint
          ./target/release/hecate-arch validate
      
      - name: Check versions
        run: ./target/release/hecate-dev version check
      
      - name: Security audit
        run: ./target/release/hecate-deps audit
```

## Development Workflow

### 1. Setup

```bash
# Clone repository
git clone https://github.com/Arakiss/hecate-os.git
cd hecate-os

# Build development tools
cd rust
cargo build --release

# Install git hooks
../target/release/hecate-hooks install
```

### 2. Making Changes

```bash
# Create feature branch
git checkout -b feat/gpu-monitoring

# Make changes...

# Check your work
hecate-dev check
hecate-lint --fix

# Create commit
hecate-dev commit create -t feat -s gpu -m "add temperature monitoring"
```

### 3. Releasing

```bash
# Check everything is ready
hecate-dev check
hecate-arch validate
hecate-deps audit

# Create release
hecate-dev release create --version 0.2.0

# Push changes
git push && git push --tags
```

## Configuration

### `.hecate/dev.toml`

```toml
[checks]
enabled = ["structure", "imports", "licenses", "todos", "dependencies", "ports"]
auto_fix = true

[version]
sync_files = ["VERSION", "rust/Cargo.toml", "hecate-dashboard/package.json"]

[commit]
types = ["feat", "fix", "docs", "style", "refactor", "perf", "test", "chore"]
max_subject_length = 72
require_scope = false

[release]
skip_tests = false
skip_changelog = false
tag_prefix = "v"
```

## Troubleshooting

### Permission Denied

```bash
chmod +x target/release/hecate-*
```

### Git Hooks Not Running

```bash
hecate-hooks install --force
```

### Version Mismatch

```bash
hecate-dev version sync
```

### Circular Dependencies

```bash
hecate-arch cycles
```

## Contributing

When contributing to HecateOS development tools:

1. Follow the same conventions as the main project
2. Add tests for new functionality
3. Update this documentation
4. Use the tools themselves during development

## License

MIT - Part of HecateOS project