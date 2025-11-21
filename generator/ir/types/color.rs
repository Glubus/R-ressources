use crate::generator::ir::types::ResourceType;
use crate::generator::ir::{
    ResourceKey, ResourceKind, ResourceNode, ResourceOrigin,
    ResourceValue,
};
use crate::generator::parsing::{ParsedResource, ScalarValue};
use crate::generator::utils::sanitize_identifier;

pub struct ColorType;

impl ResourceType for ColorType {
    fn name(&self) -> &'static str {
        "color"
    }

    fn xml_tags(&self) -> &'static [&'static str] {
        &["color"]
    }

    fn resource_kind(&self) -> ResourceKind {
        ResourceKind::Color
    }

    fn build_node(
        &self,
        parsed: &ParsedResource,
        origin: ResourceOrigin,
    ) -> Option<ResourceNode> {
        if let ScalarValue::Color(value) = &parsed.value {
            Some(ResourceNode {
                kind: ResourceKind::Color,
                value: ResourceValue::Color(value.clone()),
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
        if let ResourceValue::Color(value) = &node.value {
            let pad = " ".repeat(indent);
            let const_name =
                sanitize_identifier(&key.name).to_uppercase();
            let escaped = value.escape_debug();
            Some(format!("{pad}pub const {const_name}: &str = \"{escaped}\";\n"))
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
        let handler = ColorType;
        assert_eq!(handler.name(), "color");
    }

    // Test xml_tags
    #[test]
    fn test_handler_xml_tags() {
        let handler = ColorType;
        let tags = handler.xml_tags();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&"color"));
    }

    // Test resource_kind
    #[test]
    fn test_handler_resource_kind() {
        let handler = ColorType;
        assert_eq!(handler.resource_kind(), ModelResourceKind::Color);
    }

    // Test build_node with valid color
    #[test]
    fn test_build_node_valid_color() {
        let handler = ColorType;
        let parsed = ParsedResource {
            name: "primary_color".to_string(),
            kind: AstResourceKind::Color,
            value: ScalarValue::Color("#FF0000".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin.clone()).unwrap();
        assert_eq!(result.kind, ModelResourceKind::Color);
        assert!(matches!(
            result.value,
            ResourceValue::Color(ref s) if s == "#FF0000"
        ));
        // Verify origin is set (can't compare directly as ResourceOrigin doesn't implement PartialEq)
        assert_eq!(result.origin.file, origin.file);
    }

    // Test build_node with different color formats
    #[test]
    fn test_build_node_rgb_color() {
        let handler = ColorType;
        let parsed = ParsedResource {
            name: "bg_color".to_string(),
            kind: AstResourceKind::Color,
            value: ScalarValue::Color("rgb(255, 0, 0)".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin).unwrap();
        assert!(matches!(
            result.value,
            ResourceValue::Color(ref s) if s == "rgb(255, 0, 0)"
        ));
    }

    // Test build_node with named color
    #[test]
    fn test_build_node_named_color() {
        let handler = ColorType;
        let parsed = ParsedResource {
            name: "text_color".to_string(),
            kind: AstResourceKind::Color,
            value: ScalarValue::Color("red".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin).unwrap();
        assert!(matches!(
            result.value,
            ResourceValue::Color(ref s) if s == "red"
        ));
    }

    // Test build_node with invalid value (not a color)
    #[test]
    fn test_build_node_invalid_value() {
        let handler = ColorType;
        let parsed = ParsedResource {
            name: "not_color".to_string(),
            kind: AstResourceKind::Color,
            value: ScalarValue::Text("not a color".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test build_node with bool value (wrong type)
    #[test]
    fn test_build_node_wrong_type() {
        let handler = ColorType;
        let parsed = ParsedResource {
            name: "not_color".to_string(),
            kind: AstResourceKind::Bool,
            value: ScalarValue::Bool(true),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test emit_rust for color
    #[test]
    fn test_emit_rust_color() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "primary_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("#FF0000".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const PRIMARY_COLOR: &str"));
        assert!(result.contains("#FF0000"));
    }

    // Test emit_rust with different color formats
    #[test]
    fn test_emit_rust_rgb_color() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "bg_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("rgb(255, 0, 0)".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const BG_COLOR: &str"));
        assert!(result.contains("rgb(255, 0, 0)"));
    }

    // Test emit_rust with named color
    #[test]
    fn test_emit_rust_named_color() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "text_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("blue".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const TEXT_COLOR: &str"));
        assert!(result.contains("blue"));
    }

    // Test emit_rust with color containing special characters
    #[test]
    fn test_emit_rust_color_with_special_chars() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "special_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("#FF\"test\"".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const SPECIAL_COLOR: &str"));
        // The value should be escaped (escape_debug escapes quotes)
        // The escaped version should not contain raw quotes
        assert!(result.contains("#FF"));
        // Verify it's a valid string literal (contains quotes)
        assert!(result.contains("\""));
    }

    // Test emit_rust with wrong value type
    #[test]
    fn test_emit_rust_wrong_value_type() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "not_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Bool(true),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4);
        assert!(result.is_none());
    }

    // Test emit_rust with different indentation
    #[test]
    fn test_emit_rust_indentation() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "test_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("#000000".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 8).unwrap();
        // Should have 8 spaces of indentation
        assert!(result.starts_with("        pub const"));
    }

    // Test emit_rust with namespaced key
    #[test]
    fn test_emit_rust_namespaced() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec!["ui".to_string(), "theme".to_string()],
            name: "primary".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("#FF0000".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const PRIMARY: &str"));
        assert!(result.contains("#FF0000"));
    }

    // Test emit_rust with empty color value
    #[test]
    fn test_emit_rust_empty_color() {
        let handler = ColorType;
        let key = ResourceKey {
            namespace: vec![],
            name: "empty_color".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Color,
            value: ResourceValue::Color("".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const EMPTY_COLOR: &str"));
        assert!(result.contains("= \"\""));
    }
}
