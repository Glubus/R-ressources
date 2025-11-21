//! Analysis and validation of resource graph.
//!
//! This module performs validations on the resource graph, including:
//! - Duplicate detection (with configurable warnings/errors)
//! - Reference resolution (future)
//! - Interpolation analysis (future)
//!
//! All validations return structured `AnalysisResult` with separate warnings and errors.

use crate::generator::ir::{ResourceGraph, ResourceKey};

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are used in Display/Error implementations
pub struct AnalysisError {
    pub message: String,
    pub key: Option<ResourceKey>,
}

impl AnalysisError {
    pub fn new(
        message: impl Into<String>,
        key: Option<ResourceKey>,
    ) -> Self {
        Self {
            message: message.into(),
            key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub message: String,
    pub key: Option<ResourceKey>,
}

impl AnalysisWarning {
    pub fn new(
        message: impl Into<String>,
        key: Option<ResourceKey>,
    ) -> Self {
        Self {
            message: message.into(),
            key,
        }
    }
}

#[derive(Debug, Default)]
pub struct AnalysisResult {
    pub warnings: Vec<AnalysisWarning>,
    pub errors: Vec<AnalysisError>,
}

impl AnalysisResult {
    #[allow(dead_code)] // Reserved for future use
    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty() && self.errors.is_empty()
    }
}

/// Validation options
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidationOptions {
    /// If true, duplicate warnings become errors
    pub treat_duplicates_as_errors: bool,
}

/// Validates the resource graph and returns warnings and errors found.
///
/// Currently checks:
/// - Duplicates (same key defined multiple times) → warnings (or errors if option enabled)
#[allow(dead_code)] // Reserved for future use
pub fn validate(graph: &ResourceGraph) -> AnalysisResult {
    validate_with_options(graph, ValidationOptions::default())
}

/// Validates the graph with custom options
pub fn validate_with_options(
    graph: &ResourceGraph,
    options: ValidationOptions,
) -> AnalysisResult {
    let mut result = AnalysisResult::default();

    for (key, nodes) in graph.nodes() {
        if nodes.len() > 1 {
            // Duplicate detected - list all files where it's defined
            let file_list: Vec<String> = nodes
                .iter()
                .map(|n| n.origin.file.display().to_string())
                .collect();
            let primary_file = &file_list[0];
            let duplicate_files = &file_list[1..];
            let message = format!(
                "Duplicate resource key '{}' defined in {} files. Using '{}' (first occurrence). Duplicates in: {}",
                key.full_name(),
                nodes.len(),
                primary_file,
                duplicate_files.join(", ")
            );

            if options.treat_duplicates_as_errors {
                result.errors.push(AnalysisError::new(
                    message,
                    Some(key.clone()),
                ));
            } else {
                result.warnings.push(AnalysisWarning::new(
                    message,
                    Some(key.clone()),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::ResourceGraphBuilder;
    use crate::generator::parsing::{
        ParsedResource, ParsedResourceFile,
        ResourceKind as ParsedKind, ScalarValue,
    };
    use std::path::PathBuf;

    #[test]
    fn detects_duplicate_keys() {
        // Note: The builder currently overwrites duplicates (last one wins),
        // so we can't detect duplicates at the graph level.
        // This test verifies that the graph has only one node (the last one wins).
        let parsed1 = ParsedResourceFile::new(
            PathBuf::from("file1.xml"),
            false,
            vec![ParsedResource {
                name: "title".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("First".to_string()),
            }],
        );
        let parsed2 = ParsedResourceFile::new(
            PathBuf::from("file2.xml"),
            false,
            vec![ParsedResource {
                name: "title".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("Second".to_string()),
            }],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[
            parsed1, parsed2,
        ]);
        // Builder now collects all nodes (including duplicates)
        assert_eq!(graph.nodes().len(), 1); // One key
        assert_eq!(
            graph
                .get_all(
                    &crate::generator::ir::ResourceKey::from_path(
                        "title"
                    )
                )
                .unwrap()
                .len(),
            2
        ); // But two nodes for that key

        let result = validate(&graph);
        // Should have a warning for the duplicate
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0]
            .message
            .contains("Duplicate resource key"));
        assert!(result.warnings[0].message.contains("title"));
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn no_errors_for_unique_keys() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("file.xml"),
            false,
            vec![
                ParsedResource {
                    name: "title".to_string(),
                    kind: ParsedKind::String,
                    value: ScalarValue::Text("Hello".to_string()),
                },
                ParsedResource {
                    name: "count".to_string(),
                    kind: ParsedKind::Number,
                    value: ScalarValue::Number {
                        value: "42".to_string(),
                        explicit_type: None,
                    },
                },
            ],
        );

        let graph =
            ResourceGraphBuilder::from_parsed_files(&[parsed]);
        let result = validate(&graph);

        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());
    }
}
