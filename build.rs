/// Build script for r-resources
///
/// This build script generates Rust code from XML resource files at compile time.
/// It reads `res/values.xml` from the project root and generates type-safe constants
/// for all defined resources.
///
/// # Resource Types
///
/// Supported resource types:
/// - `<string>`: String constants
/// - `<number>`: Numeric constants (auto-detected `i64`, `f64`, or `BigDecimal`)
///     - Add `type="i32"`, `type="u32"`, `type="f32"`, etc. to force a specific Rust type
/// - `<bool>`: Boolean constants
/// - `<string-array>`: String array constants
/// - `<int-array>`: Integer array constants
/// - `<float-array>`: Float array constants
///
/// # Example
///
/// Given this `res/values.xml`:
///
/// ```xml
/// <?xml version="1.0" encoding="utf-8"?>
/// <resources>
///     <string name="app_name">My App</string>
///     <number name="max_retries">3</number>
/// </resources>
/// ```
///
/// The build script generates:
///
/// ```rust
/// pub mod r {
///     pub const APP_NAME: &str = "My App";
///     pub const MAX_RETRIES: i64 = 3;
/// }
/// ```
mod generator;

fn main() {
    generator::build();
}
