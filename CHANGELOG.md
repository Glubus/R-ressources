# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.9.0] - 2025-11-21

### 🎉 Major Refactoring

**Complete rewrite of the code generation pipeline** with improved architecture, better error handling, and enhanced maintainability.

#### Added

- **✨ New Modular Architecture**: Complete redesign with clear separation of concerns:
  - `generator/input/` - File discovery, scanning, and profile preprocessing
  - `generator/parsing/` - XML parsing into structured AST
  - `generator/ir/` - Intermediate Representation (`ResourceGraph`) for unified resource model
  - `generator/analysis/` - Validations and structured error reporting
  - `generator/generation/` - Code generation from IR (with `flat/` submodule)

- **🔍 Duplicate Detection with Warnings**:
  - Automatically detects duplicate resource keys across multiple files
  - Reports detailed warnings showing which files contain duplicates
  - Only the first occurrence is used (priority-based resolution)
  - Option to treat duplicates as errors via `R_RESOURCES_DUPLICATES_AS_ERRORS=1` environment variable
  - Generated code annotated with `#[deprecated]` for duplicate resources

- **📦 Modular Type System**:
  - Easy to add new resource types via the `ResourceType` trait
  - Each type is self-contained in `generator/ir/types/`
  - See `generator/ir/types/README.md` for documentation on adding custom types
  - Currently supports: `string`, `number`, `bool`, `color`

- **🧪 Improved Testability**:
  - Each stage of the pipeline can be tested independently
  - Clear interfaces between components
  - Comprehensive unit and integration tests

- **⚡ Enhanced Error Messages**:
  - Detailed error reporting at each stage (Loader, Parser, IR, Analysis, Generation)
  - Clear indication of file locations and context
  - Structured warnings vs errors with `AnalysisResult`

- **🎨 Improved Color API**:
  - Public fields `r`, `g`, `b`, `a` for direct access
  - New methods: `.as_hex()`, `.as_rgb()`, `.as_u32()`
  - Removed getter methods (`.r()`, `.g()`, `.b()`, `.a()`)
  - Removed `to_rgba_u32()` and `to_rgb_tuple()` (replaced by `.as_u32()` and `.as_rgb()`)

#### Changed

- **Architecture**: Replaced monolithic `codegen/` with modular `generator/` system
- **Error Handling**: Unified error types with proper error propagation through the pipeline
- **Resource Graph**: New `ResourceGraph` IR stores all nodes (including duplicates) for better analysis
- **Build System**: `build.rs` and `src/lib.rs` now use the new `generator` module
- **Profile Preprocessing**: Moved from `codegen/environment` to `generator/input/loader/profile`

#### Removed

- **Legacy `codegen/` module**: Completely removed in favor of the new `generator/` architecture
- **Old build pipeline**: Replaced with new staged pipeline (input → parsing → ir → analysis → generation)

#### Technical Details

- **Pipeline Stages**:
  1. `input::load_resources()` - Discovers and loads XML files with profile filtering
  2. `parsing::parse_raw_files()` - Parses XML into `ParsedResourceFile` AST
  3. `ir::ResourceGraphBuilder` - Builds unified `ResourceGraph` from parsed files
  4. `analysis::validate()` - Validates graph and reports warnings/errors
  5. `generation::emit()` - Generates Rust code from validated graph

- **Backward Compatibility**: API remains 100% compatible - no breaking changes for users

### Migration Notes

- **No code changes required**: The refactoring is internal only
- **Same API**: `r_resources::build()` and `r_resources::include_resources!()` work exactly as before
- **New advanced API**: `r_resources::build_with_plan()` for custom build configurations
- **Module renamed**: Internal `codegen_v2` renamed to `generator` (not exposed in public API)

## [0.8.0] - Previous versions

### Added
- **Unified `<number>` resources**: a single XML tag now covers integers, floats, and huge literals
  - Whole numbers that fit in `i64` stay `i64`
  - Decimal literals use `f64`
  - Very large values automatically become `LazyLock<BigDecimal>` (no precision loss)
- **Explicit numeric types**: add `type="i32"`, `type="u32"`, `type="f32"`, etc. to force the generated constant type, with compile-time validation
- **`r_tests::` namespace**: XML files under `res/tests/` are automatically exposed during `cargo test`, giving apps isolated fixtures for edge cases. Opt-in outside tests via `R_RESOURCES_INCLUDE_TESTS=1` or the new `build_with_options`.
- **Big number fixtures**: sample resources and tests exercise high-precision cases

### Changed
- Legacy `<int>` and `<float>` tags now feed the same number pipeline (still parsed for backwards compatibility)
- Generated code re-exports large numbers instead of duplicating constants, ensuring references always share the same storage
- Documentation and examples now use `<number>` and mention the BigDecimal fallback

## [0.7.6] - 2025-11-06

### Fixed
- **Float formatting**: Fixed issue where float values like `3.00` were incorrectly formatted as integers in generated code
  - Floats now always include a decimal point (e.g., `3.0` instead of `3`) to prevent type errors
  - Applies to both single floats and float arrays

## [0.7.0] - 2025-11-05

### Added
- **Kotlin-style nested modules in `r::`**: Flat module now uses nested namespace structure
  - Before: `r::AUTH_ERRORS_INVALID_CREDENTIALS`
  - After: `r::auth::errors::INVALID_CREDENTIALS` (much more readable!)
  - Matches Kotlin/Android `R.string.auth.title` pattern
  - Resources at root level: `r::APP_NAME`, `r::MAX_RETRIES`
  - Namespaced resources: `r::auth::TITLE`, `r::ui::colors::PRIMARY`

### Changed
- Flat module `r::` now generates nested modules instead of flat aliases
- Structure reflects XML namespace hierarchy
- More intuitive and easier to navigate
- Better autocomplete support in IDEs

### Technical
- Refactored `flat.rs` to build namespace tree structure
- Nested module generation matches type-organized modules
- All tests updated to demonstrate new syntax

## [0.6.0] - 2025-11-05

### Added
- **String interpolation**: Support for interpolated strings with embedded references
  - Strings like `"Welcome to @string/app_name!"` are resolved at build-time
  - References are recursively resolved into final string literals
  - No runtime concatenation - all resolved at compile time
- **Template functions**: Generate reusable functions with typed parameters
  - Define templates in XML: `<string name="greeting" template="Hello {name}, you have {count} messages!">`
  - Parameters with types: `<param name="name" type="string"/>`, `<param name="count" type="int"/>`
  - Generated as Rust functions: `r::greeting(name: &str, count: i64) -> String`
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
- Generated Rust modules reflect the namespace hierarchy: `r::auth::errors::INVALID_CREDENTIALS`
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

