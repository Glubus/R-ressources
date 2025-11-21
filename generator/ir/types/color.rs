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
