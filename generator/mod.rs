//! Code generation pipeline for r-resources.
//!
//! This module implements a staged architecture for generating Rust code from XML resource files.
//! The pipeline consists of five main stages:
//!
//! 1. **Input** (`input/`) - File discovery, scanning, and profile preprocessing
//! 2. **Parsing** (`parsing/`) - XML parsing into structured AST
//! 3. **IR** (`ir/`) - Intermediate Representation (`ResourceGraph`) for unified resource model
//! 4. **Analysis** (`analysis/`) - Validations and structured error reporting
//! 5. **Generation** (`generation/`) - Code generation from IR
//!
//! Each stage is independently testable and has clear interfaces, making the system
//! maintainable and extensible.

pub mod analysis;
pub mod generation;
pub mod input;
pub mod ir;
pub mod parsing;
pub mod pipeline;
pub mod utils;

pub use input::BuildPlan;

#[derive(Debug)]
pub enum BuildError {
    Pipeline(pipeline::PipelineError),
    Analysis(Vec<analysis::AnalysisError>),
    Generation(Vec<analysis::AnalysisError>),
}

impl std::fmt::Display for BuildError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Pipeline(err) => write!(f, "{err}"),
            Self::Analysis(errors) | Self::Generation(errors) => {
                for err in errors {
                    writeln!(f, "{err:?}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for BuildError {}

#[allow(dead_code)] // Public API, may be used by consumers
pub fn build_with_plan(
    plan: &BuildPlan,
) -> Result<generation::OutputArtifacts, BuildError> {
    build_with_plan_and_options(
        plan,
        analysis::ValidationOptions::default(),
    )
}

pub fn build_with_plan_and_options(
    plan: &BuildPlan,
    validation_options: analysis::ValidationOptions,
) -> Result<generation::OutputArtifacts, BuildError> {
    let pipeline_output =
        pipeline::build_graph_with_options(plan, validation_options)
            .map_err(BuildError::Pipeline)?;

    // Print warnings
    for warning in &pipeline_output.analysis_result.warnings {
        eprintln!("warning: {}", warning.message);
    }

    // Errors stop the build
    if !pipeline_output.analysis_result.errors.is_empty() {
        return Err(BuildError::Analysis(
            pipeline_output.analysis_result.errors,
        ));
    }

    generation::emit(
        &pipeline_output.graph,
        &pipeline_output.analysis_result.warnings,
    )
    .map_err(BuildError::Generation)
}

/// Writes the generated code to `OUT_DIR/r_generated.rs`
pub fn write_generated_code(code: &str) -> std::io::Result<()> {
    use std::fs;
    use std::path::Path;

    let out_dir = std::env::var("OUT_DIR").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "OUT_DIR environment variable not set",
        )
    })?;
    let dest_path = Path::new(&out_dir).join("r_generated.rs");
    fs::write(&dest_path, code)
}

/// Main build function (equivalent to legacy `codegen::build()`)
///
/// Scans `res/` and generates code in `OUT_DIR/r_generated.rs`
pub fn build() {
    use std::path::Path;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set");
    let res_dir = Path::new(&manifest_dir).join("res");
    let tests_dir = res_dir.join("tests");

    let include_tests = tests_dir.exists()
        && (std::env::var("CARGO_CFG_TEST").is_ok()
            || std::env::var("R_RESOURCES_INCLUDE_TESTS").is_ok());

    let plan = BuildPlan {
        resources_dir: res_dir,
        tests_resources_dir: include_tests.then_some(tests_dir),
        profile: std::env::var("PROFILE")
            .unwrap_or_else(|_| "debug".to_string()),
    };

    // Check if we should treat duplicates as errors
    let treat_duplicates_as_errors =
        std::env::var("R_RESOURCES_DUPLICATES_AS_ERRORS")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

    let validation_options = analysis::ValidationOptions {
        treat_duplicates_as_errors,
    };

    match build_with_plan_and_options(&plan, validation_options) {
        Ok(artifacts) => {
            // Print warnings if any
            for warning in &artifacts.warnings {
                eprintln!("warning: {warning}");
            }
            write_generated_code(&artifacts.rust)
                .expect("Failed to write generated code");
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(path: &std::path::Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn build_with_plan_runs_pipeline_and_generation() {
        let tmp = tempdir().unwrap();
        let res_dir = tmp.path().join("res");
        write_file(
            &res_dir.join("values.xml"),
            r#"<resources>
                <string name="title">Hello</string>
                <number name="max_retries">3</number>
                <bool name="enabled">true</bool>
            </resources>"#,
        );
        let plan = BuildPlan::new(res_dir, None, "debug");
        let artifacts =
            build_with_plan(&plan).expect("build succeeds");

        // Check that R struct is generated
        assert!(artifacts.rust.contains("pub struct R"));
        assert!(artifacts.rust.contains("impl R"));

        // Check that r module is generated
        assert!(artifacts.rust.contains("pub mod r {"));

        // Check that resources are generated
        assert!(artifacts.rust.contains("pub const TITLE: &str"));
        assert!(artifacts.rust.contains("\"Hello\""));
        assert!(artifacts
            .rust
            .contains("pub const MAX_RETRIES: i64"));
        assert!(artifacts.rust.contains("pub const ENABLED: bool"));
        assert!(artifacts.rust.contains("= true"));
    }

    #[test]
    fn write_generated_code_creates_file() {
        let tmp = tempdir().unwrap();
        let out_dir = tmp.path();

        // Simuler OUT_DIR
        std::env::set_var("OUT_DIR", out_dir);

        let code = r#"
pub struct R;
pub mod r {
    pub const TEST: &str = "value";
}
"#;

        write_generated_code(code).expect("write succeeds");

        let generated_file = out_dir.join("r_generated.rs");
        assert!(generated_file.exists());

        let contents = fs::read_to_string(&generated_file).unwrap();
        assert!(contents.contains("pub struct R"));
        assert!(contents.contains("pub const TEST"));

        // Cleanup
        std::env::remove_var("OUT_DIR");
    }

    #[test]
    fn build_with_duplicates_generates_warnings() {
        let tmp = tempdir().unwrap();
        let res_dir = tmp.path().join("res");
        write_file(
            &res_dir.join("values1.xml"),
            r#"<resources><string name="title">First</string></resources>"#,
        );
        write_file(
            &res_dir.join("values2.xml"),
            r#"<resources><string name="title">Second</string></resources>"#,
        );
        let plan = BuildPlan::new(res_dir, None, "debug");
        let artifacts = build_with_plan(&plan)
            .expect("build succeeds with warnings");

        // Should have warnings about duplicates
        assert!(!artifacts.warnings.is_empty());
        assert!(artifacts
            .warnings
            .iter()
            .any(|w| w.contains("Duplicate")));

        // Code should still be generated with deprecated annotation (only first one)
        assert!(artifacts.rust.contains("pub const TITLE"));
        assert!(artifacts.rust.contains("#[deprecated"));

        // Only the first duplicate should be generated (priority)
        // The second one should NOT be generated
        assert!(artifacts.rust.contains("\"First\"")); // First file is priority
        assert!(!artifacts.rust.contains("TITLE_VALUES1")); // No suffix for duplicates
        assert!(!artifacts.rust.contains("TITLE_VALUES2")); // No suffix for duplicates
    }

    #[test]
    fn build_with_duplicates_as_errors_fails() {
        let tmp = tempdir().unwrap();
        let res_dir = tmp.path().join("res");
        write_file(
            &res_dir.join("values1.xml"),
            r#"<resources><string name="title">First</string></resources>"#,
        );
        write_file(
            &res_dir.join("values2.xml"),
            r#"<resources><string name="title">Second</string></resources>"#,
        );
        let plan = BuildPlan::new(res_dir, None, "debug");
        let options = analysis::ValidationOptions {
            treat_duplicates_as_errors: true,
        };
        let result = build_with_plan_and_options(&plan, options);

        // Should fail with error
        assert!(result.is_err());
        if let Err(BuildError::Analysis(errors)) = result {
            assert!(!errors.is_empty());
            assert!(errors
                .iter()
                .any(|e| e.message.contains("Duplicate")));
        } else {
            panic!("Expected Analysis error");
        }
    }
}
