# Justfile - Command runner for the workspace
# Install: https://github.com/casey/just
# Usage: `just <command>` or `just --list` to see all commands

# Default command - show all available commands
default:
    just --list

# Build the workspace
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run Clippy linter
clippy:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run all checks (format, clippy, test)
check-all: fmt-check clippy test
    @echo "All checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
doc:
    cargo doc --open

# Check code without building
check:
    cargo check

# Run a specific binary (usage: just run <binary-name>)
run binary:
    cargo run --bin {{ binary }}

# Run a specific binary in release mode
run-release binary:
    cargo run --release --bin {{ binary }}

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run clippy
watch-clippy:
    cargo watch -x clippy

# Install development dependencies
install-dev:
    cargo install cargo-watch cargo-deny

