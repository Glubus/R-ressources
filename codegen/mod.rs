//! Build-time code generation for resources

/// Module containing type definitions for resource values
pub mod types;

/// Module for parsing XML resource files
pub mod parser;

/// Code generation modules
pub mod generator;

/// Utility functions
pub mod utils;

/// Multi-file resource loading
pub mod multi_file;

/// Reference resolution
pub mod references;

/// Environment/profile support
pub mod environment;

use std::fs;
use std::path::Path;

/// Main entry point for the build process.
///
/// This function:
/// 1. Scans the res/ directory for all XML files
/// 2. Parses and merges resources from all files
/// 3. Generates Rust code from the parsed resources
/// 4. Writes the generated code to `OUT_DIR/r_generated.rs`
pub fn build() {
    let res_dir = Path::new("res");

    // Watch the entire res/ directory
    println!("cargo:rerun-if-changed=res");

    if !res_dir.exists() {
        eprintln!("Warning: res/ directory not found, generating empty R struct");
        write_generated_code(&generator::generate_empty_code());
        return;
    }

    match multi_file::load_all_resources(res_dir) {
        Ok(resources) => {
            // Validate references
            let ref_errors = references::validate_references(&resources);
            if !ref_errors.is_empty() {
                eprintln!("error: Reference validation failed:");
                for error in &ref_errors {
                    eprintln!("  {error}");
                }
                std::process::exit(1);
            }
            
            let code = generator::generate_code(&resources);
            write_generated_code(&code);
        }
        Err(e) => {
            eprintln!("error: Failed to load resources: {e}");
            std::process::exit(1);
        }
    }
}

/// Writes the generated code to `OUT_DIR/r_generated.rs`
fn write_generated_code(code: &str) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = Path::new(&out_dir).join("r_generated.rs");

    fs::write(&dest_path, code).expect("Failed to write generated code");
}
