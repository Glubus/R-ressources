/// Code generation for advanced types (color, url, dimension)
use crate::codegen::references;
use crate::codegen::types::ResourceValue;
use crate::codegen::utils::sanitize_identifier;
use std::fmt::Write as _;

/// Generates the color module
pub fn generate_color_module(colors: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod color {\n");
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> { children: BTreeMap<String, Node<'a>>, items: Vec<(&'a str, &'a ResourceValue)> }
    fn insert<'a>(root: &mut Node<'a>, path: &'a str, value: &'a ResourceValue) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, value)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut root: Node = Default::default();
    for (name, value) in colors { insert(&mut root, name, value); }
    fn emit_node(code: &mut String, node: &Node, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_node(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, value) in &node.items {
            let const_name = sanitize_identifier(leaf).to_uppercase();
            match *value {
                ResourceValue::Color(ref c) => {
                    let _ = writeln!(code, "{}pub const {}: &str = \"{}\";", pad, const_name, c.escape_debug());
                }
                ResourceValue::Reference { ref resource_type, ref key } => {
                    let target = references::resolve_reference_path(resource_type, key, true);
                    let _ = writeln!(code, "{}pub const {}: &str = {target};", pad, const_name);
                }
                _ => {}
            }
        }
    }
    emit_node(&mut code, &root, 4);
    code.push_str("}\n");

    // also generate typed module alongside
    let mut typed = String::from("\npub mod color_t {\n    #[allow(unused_imports)]\n    use crate::Color;\n");
    #[derive(Default)]
    struct TNode<'a> { children: BTreeMap<String, TNode<'a>>, items: Vec<(&'a str, &'a str)> }
    fn insert_t<'a>(root: &mut TNode<'a>, path: &'a str, hex: &'a str) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, hex)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut troot: TNode = Default::default();
    for (name, value) in colors {
        if let ResourceValue::Color(hex) = value { insert_t(&mut troot, name, hex); }
    }
    fn emit_tnode(code: &mut String, node: &TNode, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_tnode(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, hex) in &node.items {
            if let Some((a, r, g, b)) = parse_hex_argb(hex) {
                let _ = writeln!(code, "{}pub const {}: crate::Color = crate::Color::new({r}, {g}, {b}, {a});", pad, sanitize_identifier(leaf).to_uppercase());
            }
        }
    }
    emit_tnode(&mut typed, &troot, 4);
    typed.push_str("}\n");

    code + &typed
}

/// Generates the url module
pub fn generate_url_module(urls: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod url {\n");
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> { children: BTreeMap<String, Node<'a>>, items: Vec<(&'a str, &'a str)> }
    fn insert<'a>(root: &mut Node<'a>, path: &'a str, val: &'a str) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, val)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut root: Node = Default::default();
    for (name, value) in urls { if let ResourceValue::Url(u) = value { insert(&mut root, name, u); } }
    fn emit_node(code: &mut String, node: &Node, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_node(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, u) in &node.items { let _=writeln!(code, "{}pub const {}: &str = \"{}\";", pad, sanitize_identifier(leaf).to_uppercase(), u.escape_debug()); }
    }
    emit_node(&mut code, &root, 4);
    code.push_str("}\n");
    // typed module
    let mut typed = String::from("\npub mod url_t {\n    use crate::UrlParts;\n");
    #[derive(Default)]
    struct TNode<'a> { children: BTreeMap<String, TNode<'a>>, items: Vec<(&'a str, &'a str)> }
    let mut troot: TNode = Default::default();
    fn insert_t<'a>(root: &mut TNode<'a>, path: &'a str, val: &'a str) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, val)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    for (name, value) in urls { if let ResourceValue::Url(u) = value { insert_t(&mut troot, name, u); } }
    fn emit_tnode(code: &mut String, node: &TNode, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_tnode(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, u) in &node.items { if let Some((scheme, host, path)) = split_url_parts(u) { let _=writeln!(code, "{}pub const {}: UrlParts = UrlParts::new(\"{}\", \"{}\", \"{}\");", pad, sanitize_identifier(leaf).to_uppercase(), scheme, host, path); } }
    }
    emit_tnode(&mut typed, &troot, 4);
    typed.push_str("}\n");

    code + &typed
}

/// Generates the dimension module
pub fn generate_dimension_module(dimensions: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod dimension {\n");
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Node<'a> { children: BTreeMap<String, Node<'a>>, items: Vec<(&'a str, &'a str)> }
    fn insert<'a>(root: &mut Node<'a>, path: &'a str, val: &'a str) {
        let mut parts = path.split('/').filter(|s| !s.is_empty()).peekable();
        let mut node = root;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { node.items.push((part, val)); } else {
                let key = sanitize_identifier(part); node = node.children.entry(key).or_default();
            }
        }
    }
    let mut root: Node = Default::default();
    for (name, value) in dimensions { if let ResourceValue::Dimension(d) = value { insert(&mut root, name, d); } }
    fn emit_node(code: &mut String, node: &Node, indent: usize) {
        let pad = " ".repeat(indent);
        for (k, child) in &node.children { let _=writeln!(code, "{}pub mod {} {{", pad, k); emit_node(code, child, indent+4); let _=writeln!(code, "{}}}", pad); }
        for (leaf, d) in &node.items { let _=writeln!(code, "{}pub const {}: &str = \"{}\";", pad, sanitize_identifier(leaf).to_uppercase(), d.escape_debug()); }
    }
    emit_node(&mut code, &root, 4);
    code.push_str("}\n");
    code
}

#[allow(clippy::many_single_char_names)]
fn parse_hex_argb(s: &str) -> Option<(u8, u8, u8, u8)> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((0xFF, r, g, b))
        }
        8 => {
            let a = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let r = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let g = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let b = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some((a, r, g, b))
        }
        _ => None,
    }
}

fn split_url_parts(s: &str) -> Option<(String, String, String)> {
    let (scheme, rest) = s.split_once("://")?;
    if let Some((host, path)) = rest.split_once('/') {
        Some((scheme.to_string(), host.to_string(), format!("/{path}")))
    } else {
        Some((scheme.to_string(), rest.to_string(), "/".to_string()))
    }
}



