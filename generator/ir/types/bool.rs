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
