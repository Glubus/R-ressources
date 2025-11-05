/// Code generation for basic types (string, int, float, bool)
use crate::codegen::references;
use crate::codegen::types::{InterpolationPart, ResourceValue};
use crate::codegen::utils::sanitize_identifier;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

/// Resolves a reference to its string value, handling nested references recursively
fn resolve_string_value(
    resource_type: &str,
    key: &str,
    all_resources: &HashMap<String, Vec<(String, ResourceValue)>>,
    visited: &mut HashSet<String>,
) -> Option<String> {
    let ref_key = format!("{}:{}", resource_type, key);
    if visited.contains(&ref_key) {
        // Circular reference detected
        return None;
    }
    visited.insert(ref_key.clone());

    let result = if let Some(target_resources) = all_resources.get(resource_type) {
        if let Some((_, target_value)) = target_resources.iter().find(|(n, _)| n == key) {
            extract_string_from_value(target_value, all_resources, visited)
        } else {
            None
        }
    } else {
        None
    };

    visited.remove(&ref_key);
    result
}

/// Extracts a string value from a ResourceValue, resolving references recursively
fn extract_string_from_value(
    value: &ResourceValue,
    all_resources: &HashMap<String, Vec<(String, ResourceValue)>>,
    visited: &mut HashSet<String>,
) -> Option<String> {
    match value {
        ResourceValue::String(ref s) => Some(s.clone()),
        ResourceValue::Reference { resource_type, key } => {
            // Nested reference - resolve recursively
            resolve_string_value(resource_type, key, all_resources, visited)
        }
        ResourceValue::InterpolatedString(ref parts) => {
            // Resolve all interpolation parts
            resolve_interpolation_parts(parts, all_resources, visited)
        }
        _ => {
            // Other types - can't convert to string directly
            None
        }
    }
}

/// Resolves all parts of an interpolated string into a single string
fn resolve_interpolation_parts(
    parts: &[InterpolationPart],
    all_resources: &HashMap<String, Vec<(String, ResourceValue)>>,
    visited: &mut HashSet<String>,
) -> Option<String> {
    let mut result = String::new();
    for part in parts {
        match part {
            InterpolationPart::Text(ref text) => {
                result.push_str(text);
            }
            InterpolationPart::Reference { resource_type, key } => {
                if let Some(resolved) = resolve_string_value(resource_type, key, all_resources, visited) {
                    result.push_str(&resolved);
                } else {
                    // Reference not found or circular - use placeholder
                    result.push_str(&format!("@{}", key));
                }
            }
        }
    }
    Some(result)
}

/// Generates Rust code for a simple string constant
fn emit_string_constant(code: &mut String, const_name: &str, value: &str, indent: usize) {
    let pad = " ".repeat(indent);
    let _ = writeln!(
        code,
        "{}pub const {}: &str = \"{}\";",
        pad,
        const_name,
        value.escape_debug()
    );
}

/// Generates Rust code for a string constant that references another resource
fn emit_reference_constant(
    code: &mut String,
    const_name: &str,
    resource_type: &str,
    key: &str,
    indent: usize,
) {
    let pad = " ".repeat(indent);
    let target = references::resolve_reference_path(resource_type, key, true);
    let _ = writeln!(code, "{}pub const {}: &str = {target};", pad, const_name);
}

/// Generates Rust code for an interpolated string constant
fn emit_interpolated_constant(
    code: &mut String,
    const_name: &str,
    parts: &[InterpolationPart],
    all_resources: &HashMap<String, Vec<(String, ResourceValue)>>,
    indent: usize,
) {
    // Build the final string by resolving all references at build time
    let mut visited = HashSet::new();
    let final_string = resolve_interpolation_parts(parts, all_resources, &mut visited)
        .unwrap_or_else(|| "@invalid".to_string());
    
    emit_string_constant(code, const_name, &final_string, indent);
}

/// Generates Rust code for a template function
///
/// Creates a function that takes typed parameters and returns a formatted string.
/// Example: template "Hello {name}!" with param name: string
/// Generates: `pub fn greeting(name: &str) -> String { format!("Hello {}!", name) }`
fn emit_template_function(
    code: &mut String,
    fn_name: &str,
    template: &crate::codegen::types::Template,
    indent: usize,
) {
    use crate::codegen::types::TemplateParameterType;
    use crate::codegen::utils::sanitize_identifier;
    
    let pad = " ".repeat(indent);
    
    // Build function signature with typed parameters
    let mut params = Vec::new();
    for param in &template.parameters {
        let param_name = sanitize_identifier(&param.name);
        let rust_type = match param.param_type {
            TemplateParameterType::String => "&str",
            TemplateParameterType::Int => "i64",
            TemplateParameterType::Float => "f64",
            TemplateParameterType::Bool => "bool",
        };
        params.push(format!("{param_name}: {rust_type}"));
    }
    let params_str = params.join(", ");
    
    // Parse template string: convert {param} placeholders to format!() syntax
    let template_str = &template.template;
    let (format_str, format_args) = parse_template_placeholders(template_str, &template.parameters);
    
    // Generate function body
    // Note: format! is not const in Rust stable, so this function is not const
    let _ = writeln!(code, "{}#[must_use]", pad);
    let _ = writeln!(code, "{}pub fn {fn_name}({params_str}) -> String {{", pad);
    let format_args_str = format_args.join(", ");
    if !format_args.is_empty() {
        let _ = writeln!(
            code,
            "{}    format!(\"{format_str}\", {format_args_str})",
            pad
        );
    } else {
        // No parameters, just return the template string (with escaped braces)
        let _ = writeln!(
            code,
            "{}    \"{}\".to_string()",
            pad,
            format_str.replace('\\', "")
        );
    }
    let _ = writeln!(code, "{}}}", pad);
}

/// Parses a template string and converts {param} placeholders to format!() syntax
///
/// Returns: (format_string, format_arguments)
/// Example: "Hello {name}!" -> ("Hello {}!", ["name"])
fn parse_template_placeholders(
    template_str: &str,
    parameters: &[crate::codegen::types::TemplateParameter],
) -> (String, Vec<String>) {
    use crate::codegen::utils::sanitize_identifier;
    
    let mut format_parts = Vec::new();
    let mut format_args = Vec::new();
    let mut current_pos = 0;
    
    while let Some(start) = template_str[current_pos..].find('{') {
        let start_abs = current_pos + start;
        // Add text before placeholder (escape it properly for string literal)
        let text_before = &template_str[current_pos..start_abs];
        if !text_before.is_empty() {
            format_parts.push(text_before.escape_debug().to_string());
        }
        
        if let Some(end) = template_str[start_abs + 1..].find('}') {
            let end_abs = start_abs + 1 + end;
            let placeholder = &template_str[start_abs + 1..end_abs];
            
            // Find matching parameter and replace with format!() placeholder
            if let Some(param) = parameters.iter().find(|p| p.name == placeholder) {
                let param_name = sanitize_identifier(&param.name);
                format_args.push(param_name.clone());
                format_parts.push("{}".to_string()); // format!() uses {} placeholders
            } else {
                // Unknown placeholder, keep as literal (escape braces)
                format_parts.push(format!("\\{{{placeholder}\\}}"));
            }
            
            current_pos = end_abs + 1;
        } else {
            // Unclosed brace, treat as literal
            format_parts.push("\\{".to_string());
            current_pos = start_abs + 1;
        }
    }
    
    // Add remaining text after last placeholder
    let remaining = &template_str[current_pos..];
    if !remaining.is_empty() {
        format_parts.push(remaining.escape_debug().to_string());
    }
    
    (format_parts.join(""), format_args)
}

/// Generates the string module
pub fn generate_string_module(
    strings: &[(String, ResourceValue)],
    all_resources: &HashMap<String, Vec<(String, ResourceValue)>>,
) -> String {
    let mut code = String::from("\npub mod string {\n");

    // Build a namespace tree from names like "a/b/c"
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> {
        children: BTreeMap<String, Node<'a>>,
        items: Vec<(&'a str, &'a ResourceValue)>, // (leaf_name, value)
    }

    fn insert<'a>(root: &mut Node<'a>, path: &'a str, value: &'a ResourceValue) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                node.items.push((part, value));
            } else {
                let key = sanitize_identifier(part);
                node = node.children.entry(key).or_default();
            }
        }
    }

    let mut root: Node = Default::default();
    for (name, value) in strings {
        insert(&mut root, name, value);
    }

    fn emit_node(
        code: &mut String,
        node: &Node,
        indent: usize,
        all_resources: &HashMap<String, Vec<(String, ResourceValue)>>,
    ) {
        let pad = " ".repeat(indent);
        for (mod_name, child) in &node.children {
            let _ = writeln!(code, "{}pub mod {} {{", pad, mod_name);
            emit_node(code, child, indent + 4, all_resources);
            let _ = writeln!(code, "{}}}", pad);
        }
        for (leaf, value) in &node.items {
            let const_name = sanitize_identifier(leaf).to_uppercase();
            match *value {
                ResourceValue::String(ref s) => {
                    emit_string_constant(code, &const_name, s, pad.len());
                }
                ResourceValue::Reference { ref resource_type, ref key } => {
                    emit_reference_constant(code, &const_name, resource_type, key, pad.len());
                }
                ResourceValue::InterpolatedString(ref parts) => {
                    emit_interpolated_constant(code, &const_name, parts, all_resources, pad.len());
                }
                ResourceValue::Template(ref template) => {
                    // For templates, use lowercase function name (Rust convention)
                    let fn_name = sanitize_identifier(leaf).to_lowercase();
                    emit_template_function(code, &fn_name, template, pad.len());
                }
                _ => {}
            }
        }
    }

    emit_node(&mut code, &root, 4, all_resources);

    code.push_str("}\n");
    code
}

/// Generates the int module
pub fn generate_int_module(ints: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod int {\n");
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> { children: BTreeMap<String, Node<'a>>, items: Vec<(&'a str, i64)> }
    fn insert<'a>(root: &mut Node<'a>, path: &'a str, val: i64) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, val)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut root: Node = Default::default();
    for (name, value) in ints { if let ResourceValue::Int(i) = value { insert(&mut root, name, *i); } }
    fn emit_node(code: &mut String, node: &Node, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_node(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, v) in &node.items { let _=writeln!(code, "{}pub const {}: i64 = {};", pad, sanitize_identifier(leaf).to_uppercase(), v); }
    }
    emit_node(&mut code, &root, 4);
    code.push_str("}\n");
    code
}

/// Generates the float module
pub fn generate_float_module(floats: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod float {\n");
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> { children: BTreeMap<String, Node<'a>>, items: Vec<(&'a str, f64)> }
    fn insert<'a>(root: &mut Node<'a>, path: &'a str, val: f64) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, val)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut root: Node = Default::default();
    for (name, value) in floats { if let ResourceValue::Float(f) = value { insert(&mut root, name, *f); } }
    fn emit_node(code: &mut String, node: &Node, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_node(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, v) in &node.items { let _=writeln!(code, "{}pub const {}: f64 = {};", pad, sanitize_identifier(leaf).to_uppercase(), v); }
    }
    emit_node(&mut code, &root, 4);
    code.push_str("}\n");
    code
}

/// Generates the bool module
pub fn generate_bool_module(bools: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod bool {\n");
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> { children: BTreeMap<String, Node<'a>>, items: Vec<(&'a str, bool)> }
    fn insert<'a>(root: &mut Node<'a>, path: &'a str, val: bool) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, val)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut root: Node = Default::default();
    for (name, value) in bools { if let ResourceValue::Bool(b) = value { insert(&mut root, name, *b); } }
    fn emit_node(code: &mut String, node: &Node, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_node(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, v) in &node.items { let _=writeln!(code, "{}pub const {}: bool = {};", pad, sanitize_identifier(leaf).to_uppercase(), v); }
    }
    emit_node(&mut code, &root, 4);
    code.push_str("}\n");
    code
}

