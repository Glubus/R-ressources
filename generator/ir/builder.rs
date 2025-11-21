use crate::generator::parsing::ParsedResourceFile;

use super::model::{ResourceGraph, ResourceKey};
use super::types::TypeRegistry;

#[derive(Default)]
pub struct ResourceGraphBuilder {
    graph: ResourceGraph,
    registry: TypeRegistry,
}

impl ResourceGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)] // Reserved for future use
    pub fn with_registry(registry: TypeRegistry) -> Self {
        Self {
            graph: ResourceGraph::default(),
            registry,
        }
    }

    pub fn from_parsed_files(
        files: &[ParsedResourceFile],
    ) -> ResourceGraph {
        let mut builder = Self::new();
        for file in files {
            builder.ingest_file(file);
        }
        builder.graph
    }

    fn ingest_file(&mut self, file: &ParsedResourceFile) {
        for resource in &file.resources {
            let key = ResourceKey::from_path(&resource.name);
            let origin = super::ResourceOrigin::new(
                file.path.clone(),
                file.is_test,
            );

            // Map ParsedKind to type name
            let type_name = match resource.kind {
                crate::generator::parsing::ResourceKind::String => {
                    "string"
                }
                crate::generator::parsing::ResourceKind::Number => {
                    "number"
                }
                crate::generator::parsing::ResourceKind::Bool => {
                    "bool"
                }
                crate::generator::parsing::ResourceKind::Color => {
                    "color"
                }
            };

            let Some(ty) = self.registry.find_by_name(type_name) else {
                continue;
            };
            let Some(node) = ty.build_node(resource, origin) else {
                continue;
            };
            let is_duplicate = self.graph.insert(key, node);
            if is_duplicate {
                // Duplicate detected - will be reported as warning in analysis
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{
        ResourceKey, ResourceKind, ResourceValue,
    };
    use crate::generator::parsing::{
        ParsedResource, ParsedResourceFile,
        ResourceKind as ParsedKind, ScalarValue,
    };
    use std::path::PathBuf;

    #[test]
    fn builds_graph_from_single_file() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("values.xml"),
            false,
            vec![ParsedResource {
                name: "auth/title".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("Login".to_string()),
            }],
        );

        let graph =
            ResourceGraphBuilder::from_parsed_files(&[parsed]);
        assert_eq!(graph.nodes().len(), 1);

        let key = ResourceKey::from_path("auth/title");
        let node = graph.get(&key).expect("node exists");
        match &node.value {
            ResourceValue::String(value) => {
                assert_eq!(value, "Login")
            }
            _ => panic!("expected String"),
        }
        assert!(node.origin.file.ends_with("values.xml"));
    }

    #[test]
    fn builds_graph_with_numbers_and_bools() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("values.xml"),
            false,
            vec![
                ParsedResource {
                    name: "max_retries".to_string(),
                    kind: ParsedKind::Number,
                    value: ScalarValue::Number {
                        value: "3".to_string(),
                        explicit_type: None,
                    },
                },
                ParsedResource {
                    name: "enabled".to_string(),
                    kind: ParsedKind::Bool,
                    value: ScalarValue::Bool(true),
                },
            ],
        );

        let graph =
            ResourceGraphBuilder::from_parsed_files(&[parsed]);
        assert_eq!(graph.nodes().len(), 2);

        let number_key = ResourceKey::from_path("max_retries");
        let number_node =
            graph.get(&number_key).expect("number node exists");
        match &number_node.value {
            ResourceValue::Number(
                crate::generator::ir::NumberValue::Int(i),
            ) => assert_eq!(*i, 3),
            _ => panic!("expected Number::Int(3)"),
        }
        assert_eq!(number_node.kind, ResourceKind::Number);

        let bool_key = ResourceKey::from_path("enabled");
        let bool_node =
            graph.get(&bool_key).expect("bool node exists");
        match &bool_node.value {
            ResourceValue::Bool(value) => assert!(value),
            _ => panic!("expected Bool"),
        }
        assert_eq!(bool_node.kind, ResourceKind::Bool);
    }
}
