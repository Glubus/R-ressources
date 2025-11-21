//! Pipeline orchestration: connects all stages of the code generation process.
//!
//! This module provides the main `build_graph` function that runs the complete pipeline:
//! input → parsing → IR → analysis → output

use crate::generator::analysis;
use crate::generator::input::{self, BuildPlan};
use crate::generator::ir::{ResourceGraph, ResourceGraphBuilder};
use crate::generator::parsing;

pub struct PipelineOutput {
    pub graph: ResourceGraph,
    pub analysis_result: analysis::AnalysisResult,
}

#[allow(dead_code)] // Reserved for future use
pub fn build_graph(
    plan: &BuildPlan,
) -> Result<PipelineOutput, PipelineError> {
    build_graph_with_options(
        plan,
        analysis::ValidationOptions::default(),
    )
}

pub fn build_graph_with_options(
    plan: &BuildPlan,
    validation_options: analysis::ValidationOptions,
) -> Result<PipelineOutput, PipelineError> {
    let raw_files = input::load_resources(plan)?;
    let parsed_files = parsing::parse_raw_files(&raw_files)?;
    let graph =
        ResourceGraphBuilder::from_parsed_files(&parsed_files);
    let analysis_result =
        analysis::validate_with_options(&graph, validation_options);

    Ok(PipelineOutput {
        graph,
        analysis_result,
    })
}

#[derive(Debug)]
pub enum PipelineError {
    Input(input::LoaderError),
    Parsing(parsing::ParserError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Input(err) => write!(f, "{err}"),
            Self::Parsing(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for PipelineError {}

impl From<input::LoaderError> for PipelineError {
    fn from(value: input::LoaderError) -> Self {
        Self::Input(value)
    }
}

impl From<parsing::ParserError> for PipelineError {
    fn from(value: parsing::ParserError) -> Self {
        Self::Parsing(value)
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
    fn pipeline_builds_graph_for_basic_strings() {
        let tmp = tempdir().unwrap();
        let res_dir = tmp.path().join("res");
        write_file(
            &res_dir.join("values.xml"),
            r#"<resources><string name="title">Hello</string></resources>"#,
        );

        let plan = BuildPlan::new(res_dir, None, "debug");
        let output = build_graph(&plan).expect("pipeline succeeds");

        assert!(output.analysis_result.errors.is_empty());
        assert!(output.analysis_result.warnings.is_empty());
        assert_eq!(output.graph.nodes().len(), 1);
        let key =
            crate::generator::ir::ResourceKey::from_path("title");
        let node = output.graph.get(&key).unwrap();
        match &node.value {
            crate::generator::ir::ResourceValue::String(value) => {
                assert_eq!(value, "Hello")
            }
            _ => panic!("expected String value"),
        }
    }
}
