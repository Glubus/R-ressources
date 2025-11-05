# Changelog

All notable changes to this project will be documented in this file.

## [0.6.0] - 2025-11-05

### Added
- **String interpolation**: Support for interpolated strings with embedded references
  - Strings like `"Welcome to @string/app_name!"` are resolved at build-time
  - References are recursively resolved into final string literals
  - No runtime concatenation - all resolved at compile time
- **Template functions**: Generate reusable functions with typed parameters
  - Define templates in XML: `<string name="greeting" template="Hello {name}, you have {count} messages!">`
  - Parameters with types: `<param name="name" type="string"/>`, `<param name="count" type="int"/>`
  - Generated as Rust functions: `string::greeting(name: &str, count: i64) -> String`
  - Support for string, int, float, and bool parameter types
- **Intelligent alias generation**: Flat module `r::` now uses minimal unique aliases
  - Before: `r::AUTH_ERRORS_INVALID_CREDENTIALS`
  - After: `r::INVALID_CREDENTIALS` (if unique)
  - Automatically prefixes only when conflicts occur
  - Much more readable and ergonomic

### Changed
- Flat module `r::` aliases are now optimized for readability
- String interpolation uses build-time resolution instead of runtime macros
- Template functions are not exported in `r::` (they're functions, not constants)

### Technical
- Parser detects `template` attribute and `<param>` children
- Generator creates `format!()`-based functions for templates
- Conflict detection algorithm for minimal alias generation
- All string interpolation happens at build-time (zero runtime cost)
- Comprehensive test coverage for interpolation and templates

### Examples
- `examples/v06_concat.rs`: Demonstrates string interpolation and template functions
- `tests/v06_concat.rs`: Tests for interpolated strings and template generation

## [0.5.0] - 2025-11-05

### Added
- **Nested namespaces**: Support for `<ns name="...">` XML tags to create hierarchical namespaces
- Resources can now be organized in nested namespaces like `auth/errors/invalid_credentials`
- Generated Rust modules reflect the namespace hierarchy: `string::auth::errors::INVALID_CREDENTIALS`
- Reference resolution supports namespaced paths: `@string/auth/title`
- Flat access `r::` module now exports namespaced resources with flattened aliases
- New example: `examples/v05_ns.rs`
- New test suite: `tests/v05_ns.rs`
- New resource file: `res/namespaces.xml` demonstrating namespace usage

### Changed
- Resource names are now qualified with namespace paths (e.g., `auth/title` instead of just `title`)
- All generator modules (basic, advanced, arrays) now generate nested module structures
- Reference resolution updated to handle paths with `/` separators
- Flat generator exports from nested modules with appropriate aliases

### Technical
- Parser tracks namespace stack during XML parsing
- Backward compatible: resources without namespaces continue to work as before
- All tests pass with new namespace support

## [0.4.0] - 2025-11-05

### Added
- Typed resource structs (preview): `Color`, `UrlParts`, `Position`, `LatLng`
- New example: `examples/v04_typed.rs`
- New tests: `tests/v04_typed.rs`

### Changed
- Internal parser/generator refactors to prepare v0.4 typed generation
- Clippy strict cleanups across codegen and tests

### Notes
- Backward-compatible: existing `&'static str` constants remain. Typed constants generation from XML will land next.

## [0.2.1] - 2025-11-05

### Changed
- Refactored test organization: moved all tests from `src/lib.rs` to `tests/` directory
- Created dedicated test files: `basic_resources.rs`, `errors.rs`, `v02_types.rs`, `concurrency.rs`
- Improved test maintainability with better separation of concerns

### Technical
- 20 tests total (18 integration tests + 2 doctests)
- Cleaner library code without embedded tests

## [0.2.0] - 2025-11-05

### Added
- **New resource types**: `bool`, `color`, `url`, `dimension`
- **Multi-file support**: Load resources from multiple XML files in `res/`
- **i18n/Locales**: Support for locale-specific resources (`values-fr.xml`, `values-en.xml`, etc.)
- **Resource references**: Use `@type/name` to reference other resources
- **Environment profiles**: Support for debug/release-specific resources
- **Validation**: Built-in validation for colors, URLs, and resource references
- Examples for new features
- Tests for all new types

### Changed
- Extended `ResourceValue` enum with new types
- Parser now scans entire `res/` directory for XML files
- Generator creates modules for each locale

## [0.1.0] - 2025-11-05

### Added
- Initial release
- XML resource parsing at build time
- Basic types: `string`, `int`, `float`, and their array variants
- Two access patterns: `r::NAME` and `type::NAME`
- Thread-safe and async-safe (all const)
- Zero runtime overhead
- CI/CD pipeline with GitHub Actions
- Comprehensive documentation and tests

