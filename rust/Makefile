# HecateOS Makefile
# Complete build and development workflow

.PHONY: all build release test clean install uninstall iso doctor help status

CARGO = cargo
INSTALL_DIR = /usr/local/bin
SYSTEMD_DIR = /etc/systemd/system

# Default target
all: release

# Help target
help:
	@echo "HecateOS Build System"
	@echo "===================="
	@echo ""
	@echo "Build targets:"
	@echo "  make build      - Build debug version"
	@echo "  make release    - Build release version (optimized)"
	@echo "  make test       - Run all tests"
	@echo "  make clean      - Clean build artifacts"
	@echo ""
	@echo "Installation:"
	@echo "  make install    - Install to system (requires sudo)"
	@echo "  make uninstall  - Remove from system (requires sudo)"
	@echo ""
	@echo "ISO Creation:"
	@echo "  make iso        - Build HecateOS ISO (downloads Ubuntu)"
	@echo "  make iso-test   - Create test ISO (no Ubuntu download)"
	@echo ""
	@echo "Development:"
	@echo "  make doctor     - Run system diagnostics"
	@echo "  make status     - Show component build status"
	@echo "  make run-daemon - Run hecated locally"
	@echo "  make run-monitor- Run monitoring dashboard"
	@echo "  make fix        - Fix common code issues"
	@echo ""

# Development build
build:
	@echo "Building HecateOS (debug)..."
	@$(CARGO) build --workspace

# Release build with native CPU optimizations  
release:
	@echo "Building HecateOS (release)..."
	@RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		$(CARGO) build --release --workspace

# Run tests
test:
	@echo "Running tests..."
	@$(CARGO) test --workspace
	@if [ -x test-complete.sh ]; then ./test-complete.sh; fi

# Clean build artifacts
clean:
	@echo "Cleaning..."
	@$(CARGO) clean
	@rm -f *.iso ubuntu-*.iso
	@rm -rf iso-extract/ /tmp/test*.iso

# Install binaries and services
install: release
	@echo "Installing HecateOS (requires sudo)..."
	@if [ -x installer/install.sh ]; then \
		sudo installer/install.sh; \
	else \
		echo "Installing core components..."; \
		sudo install -Dm755 target/release/hecate $(INSTALL_DIR)/hecate; \
		sudo install -Dm755 target/release/hecated $(INSTALL_DIR)/hecated; \
		sudo install -Dm755 target/release/hecate-monitor $(INSTALL_DIR)/hecate-monitor; \
		sudo install -Dm755 target/release/hecate-bench $(INSTALL_DIR)/hecate-bench; \
		sudo install -Dm755 target/release/hecate-dev $(INSTALL_DIR)/hecate-dev; \
		sudo install -Dm755 target/release/hecate-iso $(INSTALL_DIR)/hecate-iso; \
		echo "Installation complete."; \
	fi

# Uninstall from system
uninstall:
	@echo "Uninstalling HecateOS (requires sudo)..."
	@if [ -x installer/uninstall.sh ]; then \
		sudo installer/uninstall.sh; \
	else \
		sudo rm -f $(INSTALL_DIR)/hecate*; \
		echo "Uninstallation complete."; \
	fi

# Create HecateOS ISO
iso: release
	@echo "Creating HecateOS ISO..."
	@echo "This will download Ubuntu 24.04 if needed..."
	@./target/release/hecate-dev iso create --download 24.04 --output hecateos.iso

# Quick test ISO (no Ubuntu)
iso-test: release
	@echo "Creating test ISO..."
	@if [ -x test-iso-creation.sh ]; then \
		./test-iso-creation.sh; \
	else \
		./target/release/hecate-iso init -o test-config.toml; \
	fi

# Run system doctor
doctor: release
	@./target/release/hecate-dev doctor

# Show build status
status: 
	@./target/release/hecate-dev build status 2>/dev/null || echo "Build hecate-dev first: make build"

# Development: run daemon
run-daemon: build
	@echo "Starting hecated..."
	@RUST_LOG=info ./target/debug/hecated

# Development: run monitor
run-monitor: build
	@echo "Starting monitoring dashboard..."
	@echo "Dashboard will be available at http://localhost:9313"
	@RUST_LOG=info ./target/debug/hecate-monitor

# Fix common issues
fix:
	@echo "Fixing common issues..."
	@$(CARGO) fix --workspace --allow-dirty
	@$(CARGO) fmt --all

# Check code quality
check:
	@echo "Checking code..."
	@$(CARGO) check --workspace
	@$(CARGO) clippy --workspace -- -W clippy::all

# Install development dependencies
deps:
	@echo "Installing development dependencies..."
	@sudo apt-get update
	@sudo apt-get install -y p7zip-full build-essential curl git pkg-config libssl-dev
	@command -v cargo >/dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cross-compile for ISO inclusion
iso-build:
	@$(CARGO) build --release --target x86_64-unknown-linux-gnu

# Development run with verbose output
dev-run:
	@RUST_LOG=debug sudo $(CARGO) run --bin hecated -- --once --force

.DEFAULT_GOAL := help