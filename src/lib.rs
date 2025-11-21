//! # r-resources
//!
//! Android-style resource management for Rust with compile-time type safety.
//!
//! This library provides a build-time resource management system inspired by Android's `R` class.
//! Resources are defined in an XML file and compiled into type-safe Rust constants at build time,
//! resulting in zero runtime overhead.
//!
//! ## Quick Start
//!
//! 1. Create a `res/values.xml` file in your project root:
//!
//! ```xml
//! <?xml version="1.0" encoding="utf-8"?>
//! <resources>
//!     <string name="app_name">My Application</string>
//!     <number name="max_retries">3</number>
//!     <number name="version">1.0</number>
//! </resources>
//! ```
//!
//! 2. Include resources in your code:
//!
//! ```rust,ignore
//! use r_resources::include_resources;
//! include_resources!();
//! use r_resources::r;
//! let _ = r::APP_NAME;
//! let _ = r::MAX_RETRIES;
//! let _ = r::VERSION;
//! ```
//!
//! ## Supported Resource Types
//!
//! - **Strings**: `<string name="key">value</string>` → `r::KEY`
//! - **Numbers**: `<number name="key">value</number>` → `r::KEY` (auto-detected `i64`, `f64`, or `BigDecimal`)
//! - **String Arrays**: `<string-array name="key">...</string-array>` → `r::KEY`
//! - **Integer Arrays**: `<int-array name="key">...</int-array>` → `r::KEY`
//! - **Float Arrays**: `<float-array name="key">...</float-array>` → `r::KEY`
//!
//! ### Forcing numeric types
//!
//! Add `type="..."` to a `<number>` tag to pick an exact Rust type (e.g., `i32`, `u32`, `f32`, `f64`, or `bigdecimal`). Literals are validated at build time.
//!
//! ### Test-only resources
//!
//! Put XML files under `res/tests/` to generate a `r_tests::` namespace automatically during `cargo test`.
//! Set the environment variable `R_RESOURCES_INCLUDE_TESTS=1` or call [`build_with_options`] to include them in other builds.
//!
//! ## Features
//!
//! - **Build-time compilation**: All resources are compiled into your binary
//! - **Type-safe**: Each resource becomes a strongly-typed constant
//! - **Zero runtime cost**: Direct constant access, no parsing or lookups
//! - **Thread-safe**: All resources are `const` and can be safely accessed from any thread
//! - **Async-safe**: Works seamlessly in async contexts (tokio, async-std, etc.)
//! - **Familiar syntax**: Inspired by Android's resource system
//!
//! ## Thread Safety
//!
//! All generated resources are `const` values, making them inherently thread-safe:
//!
//! ```rust,ignore
//! use std::thread;
//! use r_resources::r;
//!
//! let handles: Vec<_> = (0..10)
//!     .map(|_| {
//!         thread::spawn(|| {
//!             // Safe to access from multiple threads
//!             println!("{}", r::APP_NAME);
//!         })
//!     })
//!     .collect();
//!
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! ```

// Reuse the same code generation pipeline as the build script so consumers can
// call `r_resources::build()` from their own build.rs
#[path = "../generator/mod.rs"]
pub mod generator;

/// Runs the code generation. Intended to be called from a consumer's build.rs.
///
/// It scans the consumer project's `res/` directory (using CARGO_MANIFEST_DIR)
/// and writes generated code to its OUT_DIR.
pub fn build() {
    generator::build();
}

/// Build plan for custom resource generation
pub use generator::input::BuildPlan;

/// Builds resources using a custom build plan (for CLI or advanced setups).
pub fn build_with_plan(
    plan: &BuildPlan,
) -> Result<
    generator::generation::OutputArtifacts,
    generator::BuildError,
> {
    generator::build_with_plan(plan)
}

/// Includes the generated resources from the build script.
///
/// This macro must be called once in your code (typically in `main.rs` or `lib.rs`)
/// to include the generated resource constants.
///
/// # Example
///
/// ```rust,ignore
/// use r_resources::include_resources;
/// include_resources!();
/// let _ = r::APP_NAME;
/// ```
#[macro_export]
macro_rules! include_resources {
    () => {
        include!(concat!(env!("OUT_DIR"), "/r_generated.rs"));
    };
}

pub use bigdecimal::BigDecimal;

/// Typed color parsed from hex (e.g., `#RRGGBB` or `#AARRGGBB`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the color as a hex string (e.g., "#FF5722" or "#AAFF5722")
    #[must_use]
    pub fn as_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.a, self.r, self.g, self.b)
        }
    }

    /// Returns the color as an RGB tuple (r, g, b)
    #[must_use]
    pub const fn as_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Returns the color as a u32 in ARGB format
    #[must_use]
    pub const fn as_u32(&self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }
}

/// Typed URL parts split at build-time.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UrlParts {
    scheme: &'static str,
    host: &'static str,
    path: &'static str,
}

impl UrlParts {
    #[must_use]
    pub const fn new(
        scheme: &'static str,
        host: &'static str,
        path: &'static str,
    ) -> Self {
        Self { scheme, host, path }
    }
    #[must_use]
    pub const fn scheme(&self) -> &'static str {
        self.scheme
    }
    #[must_use]
    pub const fn host(&self) -> &'static str {
        self.host
    }
    #[must_use]
    pub const fn path(&self) -> &'static str {
        self.path
    }
}

/// 2D position.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }
    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }
    /// Calculates the Euclidean distance to another position.
    ///
    /// This method is not `const` because it uses `f64::hypot()` which performs
    /// floating-point operations (including `sqrt`) that are not available in const contexts.
    #[must_use]
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }
}

/// Geographic coordinates.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LatLng {
    lat: f64,
    lng: f64,
}

impl LatLng {
    #[must_use]
    pub const fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
    #[must_use]
    pub const fn lat(&self) -> f64 {
        self.lat
    }
    #[must_use]
    pub const fn lng(&self) -> f64 {
        self.lng
    }
}
