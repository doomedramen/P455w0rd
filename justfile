# P455w0rd Password Generator - Justfile
# Run with: just <command>

# Default recipe - show help
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode (optimized)
release:
    cargo build --release

# Run the project with sample arguments
run *args:
    cargo run -- {{args}}

# Run tests
test:
    cargo test

# Run clippy (Rust linter)
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt --check

# Run all quality checks (lint, format check, test)
check: lint fmt-check test

# Clean build artifacts
clean:
    cargo clean

# Install the binary to ~/.cargo/bin
install:
    cargo install --path .

# Generate documentation
doc:
    cargo doc --open

# Show help for the built binary
help: release
    ./target/release/p455w0rd --help

# Quick test with sample words
demo: release
    ./target/release/p455w0rd pikl test 123 --quick --limit 20

# Test append functionality
test-append: release
    echo "=== Before ===" > test_append.txt
    ./target/release/p455w0rd hello world --quick --limit 5 --output test_append.txt --append
    echo "=== Results ===" && cat test_append.txt && rm test_append.txt

# Benchmark quick vs full mode
benchmark: release
    @echo "Testing quick mode (limited)..."
    time ./target/release/p455w0rd hello world test --quick --limit 10000 --quiet --output /tmp/quick.txt
    @echo "\nTesting full mode (limited)..."
    time ./target/release/p455w0rd hello world test --limit 10000 --quiet --output /tmp/full.txt
    @echo "\nQuick mode generated:" && wc -l /tmp/quick.txt
    @echo "Full mode generated:" && wc -l /tmp/full.txt
    @rm -f /tmp/quick.txt /tmp/full.txt

# Check for security issues with cargo-audit (install with: cargo install cargo-audit)
audit:
    cargo audit

# Update dependencies
update:
    cargo update

# Show project statistics
stats:
    @echo "Lines of Rust code:"
    @find src -name "*.rs" -exec wc -l {} + | tail -1
    @echo "\nDependencies:"
    @cargo tree --depth 1

# Full project setup (format, lint, test, build)
setup: fmt lint test release
    @echo "Project is ready to use!"