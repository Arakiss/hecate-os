# HecateOS Testing Guide

## Overview

This guide explains how to test HecateOS components locally, both individually and end-to-end.

## Quick Start

```bash
# Run all tests
./test-e2e.sh

# Run tests in Docker (recommended)
docker-compose -f docker-compose.test.yml up
```

## Prerequisites

### Local Testing
- Rust 1.75+ with cargo
- SQLite3 development libraries
- OpenSSL development libraries
- (Optional) NVIDIA/AMD drivers for GPU testing

### Docker Testing
- Docker 20.10+
- Docker Compose v2+

## Component Testing

### 1. Core Components

```bash
# Test hardware detection
cargo test -p hecate-core

# Run with verbose output
RUST_LOG=debug cargo run -p hecate-core --example detect
```

### 2. System Daemon

```bash
# Test daemon functionality
cargo test -p hecate-daemon

# Run daemon in test mode
sudo cargo run -p hecate-daemon -- --test-mode
```

### 3. GPU Management

```bash
# Test GPU detection and management
cargo test -p hecate-gpu

# Run GPU tests (requires GPU)
cargo test -p hecate-gpu --features gpu-tests -- --test-threads=1
```

### 4. Package Manager

```bash
# Test package management
cargo test -p hecate-pkg

# Test database operations
TEST_DB=/tmp/test.db cargo test -p hecate-pkg -- --nocapture

# CLI testing
cargo run -p hecate-pkg -- --help
cargo run -p hecate-pkg -- search test
```

### 5. Update System

```bash
# Test update system (without real kernel modules)
cargo test -p hecate-update --features test-mode

# Mock update server
cd test-fixtures/updates
python3 -m http.server 8080 &
cargo run -p hecate-update -- check
```

### 6. Monitoring

```bash
# Start monitor server
cargo run -p hecate-monitor &

# Test with curl
curl http://localhost:9313/metrics
curl http://localhost:9313/ws

# Or open in browser
xdg-open http://localhost:9313
```

### 7. Benchmarking

```bash
# Run benchmarks
cargo run -p hecate-bench -- --help
cargo run -p hecate-bench -- cpu --quick
cargo run -p hecate-bench -- memory --quick
```

## Integration Testing

### End-to-End Test Script

The `test-e2e.sh` script runs a complete test suite:

1. **Build Tests** - Verifies all components compile
2. **Unit Tests** - Runs unit tests for each component
3. **Integration Tests** - Tests component interactions
4. **CLI Tests** - Verifies CLI commands work
5. **Database Tests** - Tests persistence layer
6. **Documentation Tests** - Ensures docs build

```bash
# Run full test suite
./test-e2e.sh

# Run with verbose output
RUST_LOG=debug ./test-e2e.sh
```

### Docker-based Testing

For isolated testing environment:

```bash
# Start test environment
docker-compose -f docker-compose.test.yml up

# Run specific component tests
docker-compose -f docker-compose.test.yml run test-env \
  cargo test -p hecate-pkg

# Interactive testing shell
docker-compose -f docker-compose.test.yml run test-env bash
```

## Mock Data and Fixtures

### Creating Test Fixtures

```bash
# Create mock update files
mkdir -p test-fixtures/updates
cat > test-fixtures/updates/manifest.json <<EOF
{
  "updates": [
    {
      "id": "kernel-6.8.0",
      "type": "kernel",
      "version": "6.8.0",
      "size": 104857600,
      "url": "http://localhost:8080/kernel-6.8.0.tar.zst"
    }
  ]
}
EOF
```

### Mock Hardware Profiles

```bash
# Test with different hardware profiles
HECATE_TEST_PROFILE=ai_flagship cargo test
HECATE_TEST_PROFILE=standard cargo test
```

## Performance Testing

### Benchmarking

```bash
# Run criterion benchmarks
cargo bench --workspace

# Profile with flamegraph (requires cargo-flamegraph)
cargo flamegraph --package hecate-core --bin detect
```

### Load Testing

```bash
# Test monitor under load
for i in {1..100}; do
  curl http://localhost:9313/metrics &
done
wait
```

## Debugging

### Enable Debug Logging

```bash
# Set log level
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run with debug output
cargo run -p hecate-daemon
```

### Using GDB

```bash
# Compile with debug symbols
cargo build --workspace

# Run with GDB
gdb target/debug/hecate-daemon
(gdb) run --test-mode
```

### Memory Leak Detection

```bash
# Using valgrind
valgrind --leak-check=full \
  target/debug/hecate-monitor

# Using heaptrack
heaptrack target/debug/hecate-daemon
```

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libsqlite3-dev
      
      - name: Run tests
        run: ./test-e2e.sh
```

## Test Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html

# Open report
xdg-open tarpaulin-report.html
```

## Common Issues

### Issue: `libclang not found`

```bash
# Ubuntu/Debian
sudo apt-get install libclang-dev

# Fedora
sudo dnf install clang-devel

# macOS
brew install llvm
```

### Issue: `cannot find -lsqlite3`

```bash
# Ubuntu/Debian
sudo apt-get install libsqlite3-dev

# Fedora
sudo dnf install sqlite-devel
```

### Issue: GPU tests fail

```bash
# Mock GPU for testing
export HECATE_MOCK_GPU=true
cargo test -p hecate-gpu
```

## Test Matrix

| Component | Unit Tests | Integration | E2E | Coverage |
|-----------|------------|-------------|-----|----------|
| hecate-core | ✅ | ✅ | ✅ | 75% |
| hecate-daemon | ⚠️ | ✅ | ✅ | 60% |
| hecate-gpu | ✅ | ✅ | ⚠️ | 85% |
| hecate-ml | ✅ | ⚠️ | ⚠️ | 70% |
| hecate-monitor | ⚠️ | ✅ | ✅ | 65% |
| hecate-bench | ⚠️ | ✅ | ✅ | 55% |
| hecate-pkg | ✅ | ✅ | ⚠️ | 80% |
| hecate-update | ✅ | ⚠️ | ⚠️ | 75% |

Legend: ✅ Complete | ⚠️ Partial | ❌ Missing

## Contributing Tests

When adding new features:

1. Write unit tests in `src/lib.rs` or `src/module.rs`
2. Add integration tests in `tests/`
3. Update `test-e2e.sh` if needed
4. Ensure coverage remains above 70%
5. Document test scenarios in code comments

## Security Testing

### Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzer
cargo fuzz run parse_package_metadata
```

### Static Analysis

```bash
# Security audit
cargo audit

# Check for unsafe code
cargo geiger
```

## Release Testing Checklist

- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Documentation builds
- [ ] No clippy warnings
- [ ] Code coverage > 70%
- [ ] Performance benchmarks show no regression
- [ ] Security audit passes
- [ ] Manual testing on target hardware
- [ ] Docker image builds and runs