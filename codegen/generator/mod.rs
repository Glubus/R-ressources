//! Code generators for different resource types

/// Basic types (string, int, float, bool)
pub mod basic;

/// Advanced types (color, url, dimension)
pub mod advanced;

/// Array types (`string_array`, `int_array`, `float_array`)
pub mod arrays;

/// Flat access module (`r::`)
pub mod flat;

use std::collections::HashMap;

use super::types::ResourceValue;

/// Generates Rust code for all resources
///
/// # Arguments
///
/// * `resources` - `HashMap` of resource type to list of (name, value) pairs
///
/// # Returns
///
/// A String containing the generated Rust code
pub fn generate_code(resources: &HashMap<String, Vec<(String, ResourceValue)>>) -> String {
    let mut code = String::new();

    // Generate basic type modules
    if let Some(strings) = resources.get("string") {
        code.push_str(&basic::generate_string_module(strings, resources));
    }

    if let Some(ints) = resources.get("int") {
        code.push_str(&basic::generate_int_module(ints));
    }

    if let Some(floats) = resources.get("float") {
        code.push_str(&basic::generate_float_module(floats));
    }

    if let Some(bools) = resources.get("bool") {
        code.push_str(&basic::generate_bool_module(bools));
    }

    // Generate advanced type modules
    if let Some(colors) = resources.get("color") {
        code.push_str(&advanced::generate_color_module(colors));
    }

    if let Some(urls) = resources.get("url") {
        code.push_str(&advanced::generate_url_module(urls));
    }

    if let Some(dimensions) = resources.get("dimension") {
        code.push_str(&advanced::generate_dimension_module(dimensions));
    }

    // Generate array modules
    if let Some(string_arrays) = resources.get("string_array") {
        code.push_str(&arrays::generate_string_array_module(string_arrays));
    }

    if let Some(int_arrays) = resources.get("int_array") {
        code.push_str(&arrays::generate_int_array_module(int_arrays));
    }

    if let Some(float_arrays) = resources.get("float_array") {
        code.push_str(&arrays::generate_float_array_module(float_arrays));
    }

    // Generate flat r module with all resources
    code.push_str(&flat::generate_r_module(resources));

    // Generate main R struct
    code.push_str(&generate_r_struct());

    code
}

/// Generates an empty R struct (used when no resources file exists)
pub fn generate_empty_code() -> String {
    String::from(
        r"
pub struct R;

impl Default for R {
    fn default() -> Self {
        Self::new()
    }
}

impl R {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
",
    )
}

/// Generates the main R struct
fn generate_r_struct() -> String {
    String::from(
        r"
pub struct R;

impl Default for R {
    fn default() -> Self {
        Self::new()
    }
}

impl R {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
",
    )
}

