# HecateOS Rust Components Makefile

.PHONY: all build release install clean test

CARGO = cargo
INSTALL_DIR = /usr/local/bin
SYSTEMD_DIR = /etc/systemd/system

# Build all components
all: build

# Development build
build:
	$(CARGO) build --workspace

# Release build with native CPU optimizations
release:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		$(CARGO) build --release --workspace

# Install binaries and systemd service
install: release
	@echo "Installing HecateOS components..."
	install -Dm755 target/release/hecated $(INSTALL_DIR)/hecated
	install -Dm644 hecate-daemon.service $(SYSTEMD_DIR)/hecate-daemon.service
	systemctl daemon-reload
	@echo "Installation complete. Enable with: systemctl enable --now hecate-daemon"

# Run tests
test:
	$(CARGO) test --workspace

# Clean build artifacts
clean:
	$(CARGO) clean

# Cross-compile for inclusion in ISO
iso-build:
	$(CARGO) build --release --target x86_64-unknown-linux-gnu

# Development run
run:
	sudo $(CARGO) run --bin hecated -- --once --force