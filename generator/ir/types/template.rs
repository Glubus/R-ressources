use crate::generator::ir::model::{TemplateParam, TemplateParamValue};
use crate::generator::ir::types::ResourceType;
use crate::generator::ir::{
    ResourceKey, ResourceKind, ResourceNode, ResourceOrigin,
    ResourceValue,
};
use crate::generator::parsing::{ParsedResource, ScalarValue};
use crate::generator::utils::sanitize_identifier;

pub struct TemplateType;

impl ResourceType for TemplateType {
    fn name(&self) -> &'static str {
        "template"
    }

    fn xml_tags(&self) -> &'static [&'static str] {
        &["template"]
    }

    fn resource_kind(&self) -> crate::generator::ir::ResourceKind {
        ResourceKind::Template
    }

    fn build_node(
        &self,
        parsed: &ParsedResource,
        origin: ResourceOrigin,
    ) -> Option<ResourceNode> {
        match &parsed.value {
            // Templates with parameters from <template> tags
            ScalarValue::Template { text, params } => {
                // Convert ast::TemplateParam (with ScalarValue) to model::TemplateParam
                let model_params: Vec<TemplateParam> = params
                    .iter()
                    .map(|p| {
                        // Convert ScalarValue to TemplateParamValue
                        let param_value = match &p.value {
                            crate::generator::parsing::ScalarValue::Text(_) => {
                                crate::generator::ir::model::TemplateParamValue::String
                            }
                            crate::generator::parsing::ScalarValue::Number { explicit_type, .. } => {
                                crate::generator::ir::model::TemplateParamValue::Number {
                                    explicit_type: explicit_type.clone(),
                                }
                            }
                            crate::generator::parsing::ScalarValue::Bool(_) => {
                                crate::generator::ir::model::TemplateParamValue::Bool
                            }
                            crate::generator::parsing::ScalarValue::Color(_) => {
                                crate::generator::ir::model::TemplateParamValue::Color
                            }
                            _ => crate::generator::ir::model::TemplateParamValue::String,
                        };
                        TemplateParam {
                            name: p.name.clone(),
                            value: param_value,
                        }
                    })
                    .collect();
                Some(ResourceNode {
                    kind: ResourceKind::Template,
                    value: ResourceValue::Template {
                        text: text.clone(),
                        params: model_params,
                    },
                    origin,
                })
            }
            // Templates detected from placeholders in strings
            ScalarValue::Text(value) => {
                // Accept if explicitly marked as Template or if it contains placeholders
                let is_template = matches!(
                    parsed.kind,
                    crate::generator::parsing::ResourceKind::Template
                ) || contains_template_placeholders(value);
                
                if is_template {
                    Some(ResourceNode {
                        kind: ResourceKind::Template,
                        value: ResourceValue::Template {
                            text: value.clone(),
                            params: Vec::new(),
                        },
                        origin,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn emit_rust(
        &self,
        key: &ResourceKey,
        node: &ResourceNode,
        indent: usize,
    ) -> Option<String> {
        if let ResourceValue::Template { text, params } = &node.value {
            let pad = " ".repeat(indent);
            let func_name = sanitize_identifier(&key.name);
            
            // If we have named parameters, use them
            if !params.is_empty() {
                // Generate function with named parameters
                let param_defs: Vec<String> = params
                    .iter()
                    .map(|p| {
                        let rust_type = match &p.value {
                            TemplateParamValue::String => "&str",
                            TemplateParamValue::Number { explicit_type } => {
                                // Use explicit_type if provided, otherwise default to i64
                                match explicit_type.as_deref() {
                                    Some("bigdecimal") => "r_resources::BigDecimal",
                                    Some("i8") => "i8",
                                    Some("i16") => "i16",
                                    Some("i32") => "i32",
                                    Some("i64") => "i64",
                                    Some("u8") => "u8",
                                    Some("u16") => "u16",
                                    Some("u32") => "u32",
                                    Some("u64") => "u64",
                                    Some("f32") => "f32",
                                    Some("f64") => "f64",
                                    _ => "i64", // Default for numbers
                                }
                            }
                            TemplateParamValue::Bool => "bool",
                            TemplateParamValue::Color => "&str",
                        };
                        format!("{}: {}", sanitize_identifier(&p.name), rust_type)
                    })
                    .collect();
                let params_str = param_defs.join(", ");
                
                // Replace {name} with {} in format string
                let mut format_str = text.clone();
                for param in params {
                    format_str = format_str.replace(&format!("{{{}}}", param.name), "{}");
                }
                let format_escaped = format_str.escape_debug();
                
                // Generate parameter names for format! macro
                // For BigDecimal and other Display types, we can use them directly in format!
                let param_names: Vec<String> = params
                    .iter()
                    .map(|p| sanitize_identifier(&p.name))
                    .collect();
                let param_names_str = param_names.join(", ");
                
                Some(format!(
                    "{pad}pub fn {func_name}({params_str}) -> String {{\n\
                    {pad}    format!(\"{format_escaped}\", {param_names_str})\n\
                    {pad}}}\n"
                ))
            } else {
                // No parameters, check for old-style placeholders or treat as constant
                let placeholder_count = count_placeholders(text);
                if placeholder_count == 0 {
                    // No placeholders, treat as regular string (use uppercase for consts)
                    let escaped = text.escape_debug();
                    let const_name = func_name.to_uppercase();
                    Some(format!("{pad}pub const {const_name}: &str = \"{escaped}\";\n"))
                } else {
                    // Old-style placeholders %1$s, %2$d, etc.
                    let params: Vec<String> = (1..=placeholder_count)
                        .map(|i| format!("arg{i}: &str"))
                        .collect();
                    let params_str = params.join(", ");
                    
                    // Generate the format string replacement
                    let mut format_str = text.clone();
                    for i in 1..=placeholder_count {
                        // Replace %1$s, %2$s, etc. with {}
                        format_str = format_str.replace(&format!("%{i}$s"), "{}");
                        format_str = format_str.replace(&format!("%{i}$d"), "{}");
                    }
                    let format_escaped = format_str.escape_debug();
                    
                    Some(format!(
                        "{pad}pub fn {func_name}({params_str}) -> String {{\n\
                        {pad}    format!(\"{format_escaped}\", {})\n\
                        {pad}}}\n",
                        (1..=placeholder_count)
                            .map(|i| format!("arg{i}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                }
            }
        } else {
            None
        }
    }
}

/// Check if a string contains template placeholders (e.g., %1$s, %2$d)
fn contains_template_placeholders(text: &str) -> bool {
    // Pattern: % followed by number, $, and type specifier (s, d, etc.)
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Check if followed by number
            let mut has_number = false;
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    has_number = true;
                    chars.next();
                } else if next == '$' && has_number {
                    chars.next();
                    // Check for type specifier
                    if let Some(&spec) = chars.peek() {
                        if matches!(spec, 's' | 'd' | 'f' | 'x' | 'X') {
                            return true;
                        }
                    }
                    break;
                } else {
                    break;
                }
            }
        }
    }
    false
}

/// Count the number of unique placeholders in a template string
fn count_placeholders(template: &str) -> usize {
    use std::collections::HashSet;
    let mut placeholders = HashSet::new();
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut number = String::new();
            // Collect digits
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    number.push(next);
                    chars.next();
                } else if next == '$' && !number.is_empty() {
                    chars.next();
                    // Check for type specifier
                    if let Some(&spec) = chars.peek() {
                        if matches!(spec, 's' | 'd' | 'f' | 'x' | 'X') {
                            if let Ok(num) = number.parse::<usize>() {
                                placeholders.insert(num);
                            }
                        }
                    }
                    break;
                } else {
                    break;
                }
            }
        }
    }
    
    placeholders.len()
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
        let handler = TemplateType;
        assert_eq!(handler.name(), "template");
    }

    // Test xml_tags
    #[test]
    fn test_handler_xml_tags() {
        let handler = TemplateType;
        let tags = handler.xml_tags();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&"template"));
    }

    // Test resource_kind
    #[test]
    fn test_handler_resource_kind() {
        let handler = TemplateType;
        assert_eq!(handler.resource_kind(), ModelResourceKind::Template);
    }

    // Test contains_template_placeholders
    #[test]
    fn test_contains_template_placeholders() {
        assert!(contains_template_placeholders("Hello %1$s!"));
        assert!(contains_template_placeholders("Count: %1$d"));
        assert!(contains_template_placeholders("Name: %1$s, Age: %2$d"));
        assert!(!contains_template_placeholders("Hello World"));
        assert!(!contains_template_placeholders(""));
        assert!(!contains_template_placeholders("100%"));
    }

    // Test count_placeholders
    #[test]
    fn test_count_placeholders() {
        assert_eq!(count_placeholders("Hello %1$s!"), 1);
        assert_eq!(count_placeholders("Name: %1$s, Age: %2$d"), 2);
        assert_eq!(count_placeholders("A: %1$s, B: %2$s, C: %3$d"), 3);
        assert_eq!(count_placeholders("Hello World"), 0);
        assert_eq!(count_placeholders("%1$s %1$s"), 1); // Same placeholder twice
    }

    // Test build_node with template (detected from placeholders)
    #[test]
    fn test_build_node_with_template() {
        let handler = TemplateType;
        let parsed = ParsedResource {
            name: "welcome_message".to_string(),
            kind: AstResourceKind::String, // Templates can be detected in strings
            value: ScalarValue::Text("Hello %1$s!".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin.clone());
        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.kind, ModelResourceKind::Template);
        assert!(matches!(
            node.value,
            ResourceValue::Template { text: ref s, .. } if s == "Hello %1$s!"
        ));
        assert_eq!(node.origin.file, origin.file);
    }

    // Test build_node with explicit template tag
    #[test]
    fn test_build_node_with_explicit_template() {
        let handler = TemplateType;
        let parsed = ParsedResource {
            name: "welcome_message".to_string(),
            kind: AstResourceKind::Template, // Explicit template tag
            value: ScalarValue::Text("Hello %1$s!".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin.clone());
        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.kind, ModelResourceKind::Template);
        assert!(matches!(
            node.value,
            ResourceValue::Template { text: ref s, .. } if s == "Hello %1$s!"
        ));
    }

    // Test build_node without template placeholders
    #[test]
    fn test_build_node_without_template() {
        let handler = TemplateType;
        let parsed = ParsedResource {
            name: "simple_string".to_string(),
            kind: AstResourceKind::String,
            value: ScalarValue::Text("Hello World".to_string()),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test build_node with wrong value type
    #[test]
    fn test_build_node_wrong_value_type() {
        let handler = TemplateType;
        let parsed = ParsedResource {
            name: "not_template".to_string(),
            kind: AstResourceKind::Bool,
            value: ScalarValue::Bool(true),
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test emit_rust for template with single placeholder
    #[test]
    fn test_emit_rust_single_placeholder() {
        let handler = TemplateType;
        let key = ResourceKey {
            namespace: vec![],
            name: "welcome".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Template,
            value: ResourceValue::Template {
                text: "Hello %1$s!".to_string(),
                params: vec![],
            },
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub fn welcome"));
        assert!(result.contains("arg1: &str"));
        assert!(result.contains("format!"));
    }

    // Test emit_rust for template with multiple placeholders
    #[test]
    fn test_emit_rust_multiple_placeholders() {
        let handler = TemplateType;
        let key = ResourceKey {
            namespace: vec![],
            name: "message".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Template,
            value: ResourceValue::Template {
                text: "Name: %1$s, Age: %2$d".to_string(),
                params: vec![],
            },
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub fn message") || result.contains("pub fn MESSAGE"));
        assert!(result.contains("arg1: &str"));
        assert!(result.contains("arg2: &str"));
        assert!(result.contains("format!"));
    }

    // Test emit_rust for template without placeholders (edge case)
    #[test]
    fn test_emit_rust_no_placeholders() {
        let handler = TemplateType;
        let key = ResourceKey {
            namespace: vec![],
            name: "simple".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Template,
            value: ResourceValue::Template {
                text: "Hello World".to_string(),
                params: vec![],
            },
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        // For templates without placeholders, it should be a const (uppercase for consts)
        assert!(result.contains("pub const") && result.contains("SIMPLE"));
    }

    // Test emit_rust with wrong value type
    #[test]
    fn test_emit_rust_wrong_value_type() {
        let handler = TemplateType;
        let key = ResourceKey {
            namespace: vec![],
            name: "not_template".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Template,
            value: ResourceValue::String("Hello".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4);
        assert!(result.is_none());
    }

    // Test emit_rust with indentation
    #[test]
    fn test_emit_rust_indentation() {
        let handler = TemplateType;
        let key = ResourceKey {
            namespace: vec![],
            name: "test_template".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Template,
            value: ResourceValue::Template {
                text: "Hello %1$s!".to_string(),
                params: vec![],
            },
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 8).unwrap();
        assert!(result.starts_with("        pub fn"));
    }

    // Test emit_rust with namespaced key
    #[test]
    fn test_emit_rust_namespaced() {
        let handler = TemplateType;
        let key = ResourceKey {
            namespace: vec!["ui".to_string(), "messages".to_string()],
            name: "welcome".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Template,
            value: ResourceValue::Template {
                text: "Hello %1$s!".to_string(),
                params: vec![],
            },
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub fn welcome"));
    }
}

