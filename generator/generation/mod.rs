//! Code generation from IR.
//!
//! This module transforms the `ResourceGraph` into generated Rust code.
//! Currently supports:
//! - Flat module generation (`r::` namespace structure)
//!
//! Future generators can be added (e.g., hierarchical, JSON export, etc.)

mod flat;

use crate::generator::analysis::{self, AnalysisError};
use crate::generator::ir::{ResourceGraph, TypeRegistry};

pub struct OutputArtifacts {
    pub rust: String,
    pub warnings: Vec<String>,
}

pub fn emit(
    graph: &ResourceGraph,
    analysis_warnings: &[analysis::AnalysisWarning],
) -> Result<OutputArtifacts, Vec<AnalysisError>> {
    let registry = TypeRegistry::default();
    let mut rust_code = String::new();

    // Generate main R struct
    rust_code.push_str(
        r#"
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
"#,
    );

    // Generate flat r:: module with duplicate warnings
    rust_code.push_str(&flat::generate_r_module(
        graph,
        &registry,
        analysis_warnings,
    ));

    Ok(OutputArtifacts {
        rust: rust_code,
        warnings: analysis_warnings
            .iter()
            .map(|w| w.message.clone())
            .collect(),
    })
}
