# r-resources

[![CI](https://github.com/Glubus/r-resources/workflows/CI/badge.svg)](https://github.com/Glubus/r-resources/actions)
[![Crates.io](https://img.shields.io/crates/v/r-resources.svg)](https://crates.io/crates/r-resources)
[![Documentation](https://docs.rs/r-resources/badge.svg)](https://docs.rs/r-resources)
[![License](https://img.shields.io/crates/l/r-resources.svg)](https://github.com/Glubus/r-resources#license)

A Rust library inspired by Android/Kotlin's `R` system for managing resources at build time.

**Stop scattering magic numbers across 12 files!** Centralize all your constants, strings, colors, and configuration in one place. Modify them quickly without hunting through your codebase.

## What's New in v0.9.0

### 🎉 Major Refactoring

**v0.9.0** introduces a completely redesigned code generation pipeline with improved architecture, better error handling, and enhanced maintainability:

- **✨ New Modular Architecture**: Complete rewrite with clear separation of concerns:
  - `input/` - File discovery and preprocessing
  - `parsing/` - XML parsing into AST
  - `ir/` - Intermediate Representation (ResourceGraph)
  - `analysis/` - Validations and error reporting
  - `generation/` - Code generation from IR

- **🔍 Duplicate Detection with Warnings**: 
  - Automatically detects duplicate resource keys across multiple files
  - Reports warnings showing which files contain duplicates
  - Only the first occurrence is used (priority-based)
  - Option to treat duplicates as errors via `R_RESOURCES_DUPLICATES_AS_ERRORS=1`

- **📦 Modular Type System**: 
  - Easy to add new resource types via the `ResourceType` trait
  - Each type is self-contained in `ir/types/`
  - See `ir/types/README.md` for adding custom types

- **🧪 Better Testability**: 
  - Each stage of the pipeline can be tested independently
  - Clear interfaces between components
  - Comprehensive test coverage

- **⚡ Improved Error Messages**: 
  - Detailed error reporting at each stage
  - Clear indication of file locations and context
  - Structured warnings vs errors

### Migration from v0.8.x

The API remains **100% backward compatible**. No changes needed in your code! The refactoring is internal only.

## Features

- **Build Time**: Resources are compiled directly into your binary
- **Type-safe**: Strongly typed constants
- **Zero-cost**: No runtime overhead
- **Thread-safe**: All resources are `const` - safe to use in multi-threaded contexts
- **Async-safe**: Works perfectly with tokio, async-std, and other async runtimes
- **Simple**: Clear and elegant syntax
- **Centralized**: All constants in one place - modify quickly without searching 12 files
- **Framework-agnostic**: Works great with Leptos, egui, or any Rust UI framework

## Why r-resources?

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
<number name="max_retries">3</number>
<number name="timeout_ms">5000</number>
<number name="rate">0.75</number>
```

```rust
// Access anywhere, type-safe, zero-cost
use r_resources::r;
let retries = r::MAX_RETRIES;
let timeout = r::TIMEOUT_MS;
```

## Supported Types

- `string`: String values
- `number`: Automatically typed numerics (`i64`, `f64`, or `BigDecimal` for huge values)
- `bool`: Boolean values
- `color`: Color hex strings
- `url`: URL strings
- `dimension`: Dimension values with units (e.g., "16dp", "24px")
- `string-array`: String arrays
- `int-array`: Integer arrays
- `float-array`: Float arrays

> `number` literals are parsed automatically: whole numbers that fit in `i64` stay integers, decimal values use `f64`, and very large literals fall back to a `LazyLock<BigDecimal>` so you never lose precision.
> `BigDecimal` is re-exported by `r_resources`, no extra dependency needed.

### Forcing a numeric type

Need an exact Rust type? Add `type="..."` on the `<number>` tag:

```xml
<number name="max_retries" type="i32">3</number>
<number name="cache_size" type="u32">100</number>
<number name="tax_rate" type="f32">0.20</number>
```

Supported values: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, and `bigdecimal`. Literals are validated at build time so you'll get a friendly error if something doesn't fit.

### Test-only resources (`r_tests::`)

Place XML files under `res/tests/` to generate a separate `r_tests::` namespace that is automatically available when running `cargo test`:

```
res/
 ├─ values.xml
 └─ tests/
     └─ edge_cases.xml
```

```xml
<!-- res/tests/edge_cases.xml -->
<resources>
    <string name="test_only_message">Only visible in tests</string>
    <number name="test_limit" type="i32">2147483647</number>
</resources>
```

```rust
use r_resources::include_resources;
include_resources!();

#[test]
fn smoke() {
    assert_eq!(r_tests::TEST_ONLY_MESSAGE, "Only visible in tests");
}
```

By default, these resources are only compiled when `cargo test` runs (internally checking `CARGO_CFG_TEST`). To opt-in during other builds, set the env var `R_RESOURCES_INCLUDE_TESTS=1` or call `r_resources::build_with_plan` with `tests_res_dir`.

## Installation

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
r-resources = "0.9.0"
```

**Note**: `r-resources` is a build dependency, not a runtime dependency. It generates code at compile time. All XML files in the `res/` directory are automatically loaded and merged.

## Quick Start

### 1. Create your resources

Create `res/values.xml` at the root of your project:

```xml
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">My Awesome App</string>
    <number name="max_retries" type="i32">3</number>
    <number name="tax_rate">0.20</number>
    <bool name="debug_mode">true</bool>
</resources>
```

### 2. Use your resources

```rust
use r_resources::r;

fn main() {
    println!("App: {}", r::APP_NAME);
    println!("Max retries: {}", r::MAX_RETRIES);
    println!("Tax rate: {}%", r::TAX_RATE * 100.0_f32);
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

**Access via the unified `r::` module:**
```rust
use r_resources::r;
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
r::WELCOME_TITLE  // "Welcome to My Awesome App!"
r::API_URL_WITH_VERSION  // "https://api.example.com/v2"
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
r::greeting("Alice", 5)  // "Hello Alice, you have 5 messages!"
```

Supports `string`, `int`, `float`, and `bool` parameter types.

### Duplicate Detection (v0.9.0+)

When the same resource key is defined in multiple files, the system will:

1. **Report a warning** showing all files where the key is defined
2. **Use the first occurrence** (priority-based)
3. **Annotate the generated code** with `#[deprecated]` to indicate the duplicate

Example:
```xml
<!-- res/values1.xml -->
<string name="title">First</string>

<!-- res/values2.xml -->
<string name="title">Second</string>
```

**Build output:**
```
warning: Duplicate resource key 'title' defined in 2 files. Using 'values1.xml' (first occurrence). Duplicates in: values2.xml
```

**Generated code:**
```rust
#[deprecated(note = "Duplicate resource key 'title' defined in multiple files")]
#[allow(dead_code)] // WARNING: Duplicate resource - only first definition is used
pub const TITLE: &str = "First";
```

To treat duplicates as errors instead of warnings:
```bash
R_RESOURCES_DUPLICATES_AS_ERRORS=1 cargo build
```

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

## Access Pattern

```rust
use r_resources::r;

// Root level resources
r::APP_NAME
r::MAX_RETRIES

// Namespaced resources
r::auth::TITLE
r::auth::errors::INVALID_CREDENTIALS
r::ui::colors::PRIMARY
```

> Everything lives under the single `r` module—no juggling type-prefixed modules.
> Huge numeric constants are exposed as `LazyLock<BigDecimal>` (e.g. `r::HUGE_BALANCE`). Use them directly (`r::HUGE_BALANCE.to_string()`) or borrow via `&*r::HUGE_BALANCE`.

## Thread Safety

All resources are `const` values, making them completely thread-safe:

```rust
use std::thread;
use r_resources::r;

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

Run the examples to see r-resources in action:

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
git clone https://github.com/Glubus/r-resources.git
cd r-resources
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
