# r-ressources

[![CI](https://github.com/Glubus/r-ressources/workflows/CI/badge.svg)](https://github.com/Glubus/r-ressources/actions)
[![Crates.io](https://img.shields.io/crates/v/r-ressources.svg)](https://crates.io/crates/r-ressources)
[![Documentation](https://docs.rs/r-ressources/badge.svg)](https://docs.rs/r-ressources)
[![License](https://img.shields.io/crates/l/r-ressources.svg)](https://github.com/Glubus/r-ressources#license)

A Rust library inspired by Android/Kotlin's `R` system for managing resources at build time.

**Stop scattering magic numbers across 12 files!** Centralize all your constants, strings, colors, and configuration in one place. Modify them quickly without hunting through your codebase.

## Features

- **Build Time**: Resources are compiled directly into your binary
- **Type-safe**: Strongly typed constants
- **Zero-cost**: No runtime overhead
- **Thread-safe**: All resources are `const` - safe to use in multi-threaded contexts
- **Async-safe**: Works perfectly with tokio, async-std, and other async runtimes
- **Simple**: Clear and elegant syntax
- **Centralized**: All constants in one place - modify quickly without searching 12 files
- **Framework-agnostic**: Works great with Leptos, egui, or any Rust UI framework

## Why r-ressources?

### The Problem
```rust
// Magic numbers scattered everywhere 😞
const MAX_RETRIES: i64 = 3;  // main.rs
const TIMEOUT: i64 = 5000;   // api.rs
const RATE: f64 = 0.75;      // billing.rs
// ... 12 more files to update when changing a value
```

### The Solution
```xml
<!-- res/values.xml - One place to rule them all! -->
<int name="max_retries">3</int>
<int name="timeout_ms">5000</int>
<float name="rate">0.75</float>
```

```rust
// Access anywhere, type-safe, zero-cost
use r_ressources::r;
let retries = r::MAX_RETRIES;
let timeout = r::TIMEOUT_MS;
```

## Supported Types

- `string`: String values
- `int`: Integer values (i64)
- `float`: Floating-point values (f64)
- `bool`: Boolean values
- `color`: Color hex strings
- `url`: URL strings
- `dimension`: Dimension values with units (e.g., "16dp", "24px")
- `string-array`: String arrays
- `int-array`: Integer arrays
- `float-array`: Float arrays

## Installation

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
r-ressources = "0.7.1"
```

**Note**: `r-ressources` is a build dependency, not a runtime dependency. It generates code at compile time. All XML files in the `res/` directory are automatically loaded and merged.

## Quick Start

### 1. Create your resources

Create `res/values.xml` at the root of your project:

```xml
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">My Awesome App</string>
    <int name="max_retries">3</int>
    <float name="tax_rate">0.20</float>
    <bool name="debug_mode">true</bool>
</resources>
```

### 2. Use your resources

```rust
use r_ressources::r;

fn main() {
    println!("App: {}", r::APP_NAME);
    println!("Max retries: {}", r::MAX_RETRIES);
    println!("Tax rate: {}%", r::TAX_RATE * 100.0);
}
```

## Advanced Features

### Namespaces (v0.5.0+)

Organize resources hierarchically:

```xml
<resources>
    <ns name="auth">
        <string name="title">Login</string>
        <ns name="errors">
            <string name="invalid_credentials">Invalid credentials</string>
        </ns>
    </ns>
    
    <ns name="ui">
        <ns name="colors">
            <color name="primary">#3366FF</color>
        </ns>
    </ns>
</resources>
```

**Access via type-organized modules:**
```rust
use r_ressources::string;
string::auth::TITLE
string::auth::errors::INVALID_CREDENTIALS
```

**Access via Kotlin-style `r::` module:**
```rust
use r_ressources::r;
r::auth::TITLE
r::auth::errors::INVALID_CREDENTIALS
r::ui::colors::PRIMARY
```

### String Interpolation (v0.6.0+)

Resolve references at build-time:

```xml
<string name="base_url">https://api.example.com</string>
<string name="api_version">v2</string>
<string name="welcome_title">Welcome to @string/app_name!</string>
<string name="api_url_with_version">@string/base_url/@string/api_version</string>
```

**Generated:**
```rust
string::WELCOME_TITLE  // "Welcome to My Awesome App!"
string::API_URL_WITH_VERSION  // "https://api.example.com/v2"
```

All references are resolved at compile-time - no runtime concatenation!

### Template Functions (v0.6.0+)

Generate reusable functions with typed parameters:

```xml
<string name="greeting" template="Hello {name}, you have {count} messages!">
    <param name="name" type="string"/>
    <param name="count" type="int"/>
</string>
```

**Generated:**
```rust
string::greeting("Alice", 5)  // "Hello Alice, you have 5 messages!"
```

Supports `string`, `int`, `float`, and `bool` parameter types.

### Multiple Resource Files

Support for multiple XML files in the `res/` directory:

```
res/
  ├── values.xml      # Main resources
  ├── config.xml      # Configuration
  └── theme.xml       # Theme-specific resources
```

All XML files in `res/` are automatically loaded and merged at build time.

### Simulating Locales

Use namespaces to organize by language - no need for locale-specific files:

```xml
<ns name="fr">
    <string name="welcome">Bienvenue!</string>
</ns>
<ns name="en">
    <string name="welcome">Welcome!</string>
</ns>
```

```rust
// Switch based on user locale
let welcome = if locale == "fr" {
    r::fr::WELCOME
} else {
    r::en::WELCOME
};
```

## Access Patterns

### Kotlin-style Flat Access (Recommended)

```rust
use r_ressources::r;

// Root level resources
r::APP_NAME
r::MAX_RETRIES

// Namespaced resources
r::auth::TITLE
r::auth::errors::INVALID_CREDENTIALS
r::ui::colors::PRIMARY
```

### Type-Organized Access

```rust
use r_ressources::*;

// Explicit type organization
string::APP_NAME
int::MAX_RETRIES
string::auth::TITLE
color::ui::colors::PRIMARY
```

Both patterns are equally performant - choose what fits your style!

## Thread Safety

All resources are `const` values, making them completely thread-safe:

```rust
use std::thread;
use r_ressources::r;

// Safe to access from multiple threads
let handles: Vec<_> = (0..10)
    .map(|_| {
        thread::spawn(|| {
            println!("App: {}", r::APP_NAME);
        })
    })
    .collect();
```

## Performance

- **Compilation**: Resources parsed once at build time
- **Runtime**: Zero overhead - direct constant access
- **Memory**: Resources live in binary's data segment
- **Concurrency**: No locks, no synchronization needed

## Examples

Run the examples to see r-ressources in action:

```bash
# Basic usage
cargo run --example basic_usage

# New resource types
cargo run --example v02_new_types

# Resource references
cargo run --example v03_references

# Namespaces
cargo run --example v05_ns

# String interpolation and templates
cargo run --example v06_concat
```

## Philosophy

**Centralize. Type-safe. Zero-cost.**

- **Centralize**: All constants in `res/` - modify quickly without searching your codebase
- **Type-safe**: Compile-time errors catch typos and mismatches
- **Zero-cost**: Direct constant access - no runtime overhead
- **Simple**: Familiar XML syntax, elegant Rust API

Perfect for projects where you need to:
- Avoid magic numbers scattered across 12 different files
- Quickly and simply modify constants without hunting through your codebase
- Share constants across multiple modules
- Build type-safe UI applications with Leptos, egui, or any Rust framework
- Centralize all configuration in one place for easy maintenance

## Development

### Building from source

```bash
git clone https://github.com/Glubus/r-ressources.git
cd r-ressources
cargo build
cargo test
```

### Code quality

```bash
cargo fmt       # Format code
cargo clippy    # Lint code
cargo test      # Run all tests
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
