use crate::generator::ir::types::ResourceType;
use crate::generator::ir::{
    ResourceKey, ResourceKind, ResourceNode, ResourceOrigin,
    ResourceValue,
};
use crate::generator::parsing::{ParsedResource, ScalarValue};
use crate::generator::utils::sanitize_identifier;

pub struct BoolType;

impl ResourceType for BoolType {
    fn name(&self) -> &'static str {
        "bool"
    }

    fn xml_tags(&self) -> &'static [&'static str] {
        &["bool"]
    }

    fn resource_kind(&self) -> crate::generator::ir::ResourceKind {
        ResourceKind::Bool
    }

    fn build_node(
        &self,
        parsed: &ParsedResource,
        origin: ResourceOrigin,
    ) -> Option<ResourceNode> {
        if let ScalarValue::Bool(value) = &parsed.value {
            Some(ResourceNode {
                kind: ResourceKind::Bool,
                value: ResourceValue::Bool(*value),
                origin,
            })
        } else {
            None
        }
    }

    fn emit_rust(
        &self,
        key: &ResourceKey,
        node: &ResourceNode,
        indent: usize,
    ) -> Option<String> {
        if let ResourceValue::Bool(value) = &node.value {
            let pad = " ".repeat(indent);
            let const_name =
                sanitize_identifier(&key.name).to_uppercase();
            Some(format!(
                "{pad}pub const {const_name}: bool = {value};\n"
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::model::ResourceKind as ModelResourceKind;
    use crate::generator::parsing::{ResourceKind as AstResourceKind, ScalarValue};
    use std::path::PathBuf;

    // Test name
    #[test]
    fn test_handler_name() {
        let handler = BoolType;
        assert_eq!(handler.name(), "bool");
    }

    // Test xml_tags
    #[test]
    fn test_handler_xml_tags() {
        let handler = BoolType;
        let tags = handler.xml_tags();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&"bool"));
    }

    // Test resource_kind
    #[test]
    fn test_handler_resource_kind() {
        let handler = BoolType;
        assert_eq!(handler.resource_kind(), ModelResourceKind::Bool);
    }

    // Test build_node with true
    #[test]
    fn test_build_node_true() {
        let handler = BoolType;
        let parsed = ParsedResource {
            name: "enabled".to_string(),
            kind: AstResourceKind::Bool,
            value: ScalarValue::Bool(true),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin.clone()).unwrap();
        assert_eq!(result.kind, ModelResourceKind::Bool);
        assert!(matches!(result.value, ResourceValue::Bool(true)));
        // Verify origin is set (can't compare directly as ResourceOrigin doesn't implement PartialEq)
        assert_eq!(result.origin.file, origin.file);
    }

    // Test build_node with false
    #[test]
    fn test_build_node_false() {
        let handler = BoolType;
        let parsed = ParsedResource {
            name: "disabled".to_string(),
            kind: AstResourceKind::Bool,
            value: ScalarValue::Bool(false),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin).unwrap();
        assert_eq!(result.kind, ModelResourceKind::Bool);
        assert!(matches!(result.value, ResourceValue::Bool(false)));
    }

    // Test build_node with invalid value (not a bool)
    #[test]
    fn test_build_node_invalid_value() {
        let handler = BoolType;
        let parsed = ParsedResource {
            name: "not_bool".to_string(),
            kind: AstResourceKind::Bool,
            value: ScalarValue::Text("not a bool".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test build_node with color value (wrong type)
    #[test]
    fn test_build_node_wrong_type() {
        let handler = BoolType;
        let parsed = ParsedResource {
            name: "not_bool".to_string(),
            kind: AstResourceKind::Color,
            value: ScalarValue::Color("#FF0000".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test emit_rust for true
    #[test]
    fn test_emit_rust_true() {
        let handler = BoolType;
        let key = ResourceKey {
            namespace: vec![],
            name: "enabled".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Bool,
            value: ResourceValue::Bool(true),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const ENABLED: bool = true;"));
    }

    // Test emit_rust for false
    #[test]
    fn test_emit_rust_false() {
        let handler = BoolType;
        let key = ResourceKey {
            namespace: vec![],
            name: "disabled".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Bool,
            value: ResourceValue::Bool(false),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const DISABLED: bool = false;"));
    }

    // Test emit_rust with wrong value type
    #[test]
    fn test_emit_rust_wrong_value_type() {
        let handler = BoolType;
        let key = ResourceKey {
            namespace: vec![],
            name: "not_bool".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Bool,
            value: ResourceValue::Color("#FF0000".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4);
        assert!(result.is_none());
    }

    // Test emit_rust with different indentation
    #[test]
    fn test_emit_rust_indentation() {
        let handler = BoolType;
        let key = ResourceKey {
            namespace: vec![],
            name: "test_bool".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Bool,
            value: ResourceValue::Bool(true),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 8).unwrap();
        // Should have 8 spaces of indentation
        assert!(result.starts_with("        pub const"));
    }

    // Test emit_rust with namespaced key
    #[test]
    fn test_emit_rust_namespaced() {
        let handler = BoolType;
        let key = ResourceKey {
            namespace: vec!["settings".to_string(), "feature".to_string()],
            name: "enabled".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Bool,
            value: ResourceValue::Bool(true),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const ENABLED: bool = true;"));
    }

    // Test emit_rust with various bool variable names
    #[test]
    fn test_emit_rust_various_names() {
        let handler = BoolType;
        let test_cases = vec![
            ("is_enabled", "IS_ENABLED"),
            ("has_permission", "HAS_PERMISSION"),
            ("can_edit", "CAN_EDIT"),
            ("should_show", "SHOULD_SHOW"),
        ];

        for (name, expected_const) in test_cases {
            let key = ResourceKey {
                namespace: vec![],
                name: name.to_string(),
            };
            let node = ResourceNode {
                kind: ModelResourceKind::Bool,
                value: ResourceValue::Bool(true),
                origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
            };

            let result = handler.emit_rust(&key, &node, 4).unwrap();
            assert!(
                result.contains(&format!("pub const {}: bool", expected_const)),
                "Failed for name: {}",
                name
            );
        }
    }

    // Test build_node with multiple true values
    #[test]
    fn test_build_node_multiple_true() {
        let handler = BoolType;
        let test_cases = vec![
            ("feature_flag", true),
            ("debug_mode", true),
            ("verbose", true),
        ];

        for (name, value) in test_cases {
            let parsed = ParsedResource {
                name: name.to_string(),
                kind: AstResourceKind::Bool,
                value: ScalarValue::Bool(value),
            };
            let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

            let result = handler.build_node(&parsed, origin).unwrap();
            assert_eq!(result.kind, ModelResourceKind::Bool);
            assert!(matches!(result.value, ResourceValue::Bool(v) if v == value));
        }
    }

    // Test build_node with multiple false values
    #[test]
    fn test_build_node_multiple_false() {
        let handler = BoolType;
        let test_cases = vec![
            ("disabled", false),
            ("hidden", false),
            ("locked", false),
        ];

        for (name, value) in test_cases {
            let parsed = ParsedResource {
                name: name.to_string(),
                kind: AstResourceKind::Bool,
                value: ScalarValue::Bool(value),
            };
            let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

            let result = handler.build_node(&parsed, origin).unwrap();
            assert_eq!(result.kind, ModelResourceKind::Bool);
            assert!(matches!(result.value, ResourceValue::Bool(v) if v == value));
        }
    }
}
