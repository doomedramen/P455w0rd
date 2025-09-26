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
    ./target/release/p455w0rd pikl test 123 --limit 50

# Test append functionality
test-append: release
    echo "=== Before ===" > test_append.txt
    ./target/release/p455w0rd hello world --limit 20 --output test_append.txt --append
    echo "=== Results ===" && cat test_append.txt && rm test_append.txt

# Benchmark small vs large wordlist
benchmark: release
    @echo "Testing small wordlist (3 words)..."
    time ./target/release/p455w0rd hello world test --limit 5000 --quiet --output /tmp/small.txt
    @echo "\nTesting larger wordlist (5 words)..."
    time ./target/release/p455w0rd hello world test foo bar --limit 5000 --quiet --output /tmp/large.txt
    @echo "\n3-word generated:" && wc -l /tmp/small.txt
    @echo "5-word generated:" && wc -l /tmp/large.txt
    @rm -f /tmp/small.txt /tmp/large.txt

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