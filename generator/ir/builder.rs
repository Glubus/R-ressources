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
                crate::generator::parsing::ResourceKind::Template => {
                    "template"
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

    #[test]
    fn builds_graph_from_multiple_files() {
        let file1 = ParsedResourceFile::new(
            PathBuf::from("values1.xml"),
            false,
            vec![ParsedResource {
                name: "app_name".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("MyApp".to_string()),
            }],
        );
        let file2 = ParsedResourceFile::new(
            PathBuf::from("values2.xml"),
            false,
            vec![ParsedResource {
                name: "version".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("1.0.0".to_string()),
            }],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[file1, file2]);
        assert_eq!(graph.nodes().len(), 2);
    }

    #[test]
    fn handles_duplicate_keys() {
        let file1 = ParsedResourceFile::new(
            PathBuf::from("values1.xml"),
            false,
            vec![ParsedResource {
                name: "title".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("First".to_string()),
            }],
        );
        let file2 = ParsedResourceFile::new(
            PathBuf::from("values2.xml"),
            false,
            vec![ParsedResource {
                name: "title".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("Second".to_string()),
            }],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[file1, file2]);
        let key = ResourceKey::from_path("title");
        
        // Should have both nodes
        let all_nodes = graph.get_all(&key).expect("key exists");
        assert_eq!(all_nodes.len(), 2);
        
        // First node should be the primary one
        let first_node = graph.get(&key).expect("node exists");
        match &first_node.value {
            ResourceValue::String(value) => assert_eq!(value, "First"),
            _ => panic!("expected String"),
        }
        
        // Should detect duplicates
        assert!(graph.has_duplicates(&key));
    }

    #[test]
    fn builds_graph_with_colors() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("values.xml"),
            false,
            vec![ParsedResource {
                name: "primary_color".to_string(),
                kind: ParsedKind::Color,
                value: ScalarValue::Color("#FF0000".to_string()),
            }],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[parsed]);
        assert_eq!(graph.nodes().len(), 1);

        let key = ResourceKey::from_path("primary_color");
        let node = graph.get(&key).expect("node exists");
        match &node.value {
            ResourceValue::Color(value) => assert_eq!(value, "#FF0000"),
            _ => panic!("expected Color"),
        }
        assert_eq!(node.kind, ResourceKind::Color);
    }

    #[test]
    fn builds_graph_with_namespaced_resources() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("values.xml"),
            false,
            vec![
                ParsedResource {
                    name: "auth/title".to_string(),
                    kind: ParsedKind::String,
                    value: ScalarValue::Text("Login".to_string()),
                },
                ParsedResource {
                    name: "auth/error/message".to_string(),
                    kind: ParsedKind::String,
                    value: ScalarValue::Text("Invalid".to_string()),
                },
            ],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[parsed]);
        assert_eq!(graph.nodes().len(), 2);

        let key1 = ResourceKey::from_path("auth/title");
        let node1 = graph.get(&key1).expect("node exists");
        assert_eq!(key1.namespace, vec!["auth"]);
        assert_eq!(key1.name, "title");
        match &node1.value {
            ResourceValue::String(value) => assert_eq!(value, "Login"),
            _ => panic!("expected String"),
        }

        let key2 = ResourceKey::from_path("auth/error/message");
        let node2 = graph.get(&key2).expect("node exists");
        assert_eq!(key2.namespace, vec!["auth", "error"]);
        assert_eq!(key2.name, "message");
        match &node2.value {
            ResourceValue::String(value) => assert_eq!(value, "Invalid"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn builds_graph_with_test_files() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("test_values.xml"),
            true, // is_test
            vec![ParsedResource {
                name: "test_string".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("Test".to_string()),
            }],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[parsed]);
        let key = ResourceKey::from_path("test_string");
        let node = graph.get(&key).expect("node exists");
        assert!(node.origin.is_test);
    }

    #[test]
    fn builder_new_creates_default() {
        let builder = ResourceGraphBuilder::new();
        assert_eq!(builder.graph.nodes().len(), 0);
    }

    #[test]
    fn builder_with_registry() {
        let registry = TypeRegistry::default();
        let builder = ResourceGraphBuilder::with_registry(registry);
        assert_eq!(builder.graph.nodes().len(), 0);
    }

    #[test]
    fn ignores_invalid_resources() {
        // Create a file with a resource that can't be built
        // (e.g., invalid number format)
        let parsed = ParsedResourceFile::new(
            PathBuf::from("values.xml"),
            false,
            vec![
                ParsedResource {
                    name: "valid_string".to_string(),
                    kind: ParsedKind::String,
                    value: ScalarValue::Text("Valid".to_string()),
                },
                ParsedResource {
                    name: "invalid_number".to_string(),
                    kind: ParsedKind::Number,
                    value: ScalarValue::Number {
                        value: "not_a_number".to_string(),
                        explicit_type: None,
                    },
                },
            ],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[parsed]);
        // Only valid resource should be added
        assert_eq!(graph.nodes().len(), 1);
        
        let key = ResourceKey::from_path("valid_string");
        assert!(graph.get(&key).is_some());
        
        let invalid_key = ResourceKey::from_path("invalid_number");
        assert!(graph.get(&invalid_key).is_none());
    }

    #[test]
    fn builds_graph_with_mixed_types() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("values.xml"),
            false,
            vec![
                ParsedResource {
                    name: "app_name".to_string(),
                    kind: ParsedKind::String,
                    value: ScalarValue::Text("MyApp".to_string()),
                },
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
                ParsedResource {
                    name: "primary_color".to_string(),
                    kind: ParsedKind::Color,
                    value: ScalarValue::Color("#FF0000".to_string()),
                },
            ],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[parsed]);
        assert_eq!(graph.nodes().len(), 4);

        // Verify all types are present
        let string_key = ResourceKey::from_path("app_name");
        assert!(matches!(
            graph.get(&string_key).unwrap().value,
            ResourceValue::String(_)
        ));

        let number_key = ResourceKey::from_path("max_retries");
        assert!(matches!(
            graph.get(&number_key).unwrap().value,
            ResourceValue::Number(_)
        ));

        let bool_key = ResourceKey::from_path("enabled");
        assert!(matches!(
            graph.get(&bool_key).unwrap().value,
            ResourceValue::Bool(_)
        ));

        let color_key = ResourceKey::from_path("primary_color");
        assert!(matches!(
            graph.get(&color_key).unwrap().value,
            ResourceValue::Color(_)
        ));
    }

    #[test]
    fn preserves_origin_information() {
        let parsed = ParsedResourceFile::new(
            PathBuf::from("res/values/strings.xml"),
            false,
            vec![ParsedResource {
                name: "title".to_string(),
                kind: ParsedKind::String,
                value: ScalarValue::Text("Hello".to_string()),
            }],
        );

        let graph = ResourceGraphBuilder::from_parsed_files(&[parsed]);
        let key = ResourceKey::from_path("title");
        let node = graph.get(&key).expect("node exists");
        
        assert!(node.origin.file.ends_with("strings.xml"));
        assert!(!node.origin.is_test);
    }
}
