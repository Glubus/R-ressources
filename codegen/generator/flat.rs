/// Code generation for flat `r::` access module (Kotlin-style nested modules)
use std::collections::{BTreeMap, HashMap};

use crate::codegen::types::ResourceValue;
use crate::codegen::utils::sanitize_identifier;
use std::fmt::Write as _;

/// Represents a resource entry for flat module generation
struct ResourceEntry {
    resource_type: String,
    full_path: String,
    namespace_path: Vec<String>,
    leaf_name: String,
}

/// Generates a flat module `r` with nested namespace structure (Kotlin-style: r::auth::title)
pub fn generate_r_module(resources: &HashMap<String, Vec<(String, ResourceValue)>>) -> String {
    let mut code = String::from("\n/// Flat access to all resources via `r::{ns}::value` (Kotlin-style)\npub mod r {\n");

    // Collect all resources and organize by namespace
    let all_entries = collect_all_resources(resources);
    let namespace_tree = build_namespace_tree(&all_entries);
    
    // Generate nested modules
    emit_namespace_tree(&mut code, &namespace_tree, &all_entries, 4);

    code.push_str("}\n");
    
    // typed flat module
    let mut typed = String::from("\n/// Flat access for typed resources via `r_t::{ns}::value`\npub mod r_t {\n");
    let typed_entries = collect_typed_resources(resources);
    let typed_tree = build_namespace_tree(&typed_entries);
    emit_namespace_tree(&mut typed, &typed_tree, &typed_entries, 4);
    typed.push_str("}\n");

    code + &typed
}

/// Collects all resources from all types into a unified list
fn collect_all_resources(resources: &HashMap<String, Vec<(String, ResourceValue)>>) -> Vec<ResourceEntry> {
    let mut entries = Vec::new();
    let resource_types = ["string", "int", "float", "bool", "color", "url", "dimension", "string_array", "int_array", "float_array"];
    
    for resource_type in &resource_types {
        if let Some(items) = resources.get(*resource_type) {
            for (name, value) in items {
                if name.is_empty() {
                    continue;
                }
                // Skip templates - they are functions, not constants
                if matches!(value, ResourceValue::Template(_)) {
                    continue;
                }
                
                let (namespace_path, leaf_name) = split_path_to_namespace(name);
                entries.push(ResourceEntry {
                    resource_type: resource_type.to_string(),
                    full_path: name.clone(),
                    namespace_path,
                    leaf_name: leaf_name.to_string(),
                });
            }
        }
    }
    
    entries
}

/// Collects typed resources (color_t, url_t)
fn collect_typed_resources(resources: &HashMap<String, Vec<(String, ResourceValue)>>) -> Vec<ResourceEntry> {
    let mut entries = Vec::new();
    let resource_types = ["color_t", "url_t"];
    
    for resource_type in &resource_types {
        if let Some(items) = resources.get(*resource_type) {
            for (name, _value) in items {
                if name.is_empty() {
                    continue;
                }
                
                let (namespace_path, leaf_name) = split_path_to_namespace(name);
                entries.push(ResourceEntry {
                    resource_type: resource_type.to_string(),
                    full_path: name.clone(),
                    namespace_path,
                    leaf_name: leaf_name.to_string(),
                });
            }
        }
    }
    
    entries
}

/// Splits a path like "auth/errors/invalid_credentials" into (namespace_path, leaf_name)
fn split_path_to_namespace(path: &str) -> (Vec<String>, &str) {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return (Vec::new(), "");
    }
    
    if parts.len() == 1 {
        return (Vec::new(), parts[0]);
    }
    
    let leaf = parts[parts.len() - 1];
    let namespace_parts: Vec<String> = parts[..parts.len() - 1]
        .iter()
        .map(|p| sanitize_identifier(p))
        .collect();
    
    (namespace_parts, leaf)
}

/// Namespace tree node
#[derive(Default)]
struct NamespaceNode {
    children: BTreeMap<String, NamespaceNode>,
    resource_paths: Vec<String>, // Full paths of resources in this namespace
}

/// Builds a namespace tree from resource entries
fn build_namespace_tree(entries: &[ResourceEntry]) -> NamespaceNode {
    let mut root = NamespaceNode::default();
    
    for entry in entries {
        // Traverse/create namespace path using indices
        let mut current = &mut root;
        for ns_part in &entry.namespace_path {
            current = current.children.entry(ns_part.clone()).or_default();
        }
        
        // Add resource path to this namespace
        current.resource_paths.push(entry.full_path.clone());
    }
    
    root
}

/// Emits nested namespace modules recursively
fn emit_namespace_tree(
    code: &mut String,
    node: &NamespaceNode,
    entries: &[ResourceEntry],
    indent: usize,
) {
    let pad = " ".repeat(indent);
    
    // First, emit child namespaces (modules)
    for (ns_name, child) in &node.children {
        let _ = writeln!(code, "{}pub mod {} {{", pad, ns_name);
        emit_namespace_tree(code, child, entries, indent + 4);
        let _ = writeln!(code, "{}}}", pad);
    }
    
    // Then, emit resources in this namespace (pub use statements)
    for resource_path in &node.resource_paths {
        // Find the entry for this resource
        if let Some(entry) = entries.iter().find(|e| e.full_path == *resource_path) {
            let const_name = sanitize_identifier(&entry.leaf_name).to_uppercase();
            
            // Build the source path
            if entry.namespace_path.is_empty() {
                // Root level resource
                let _ = writeln!(
                    code,
                    "{}pub use crate::{}::{};",
                    pad,
                    entry.resource_type,
                    const_name
                );
            } else {
                // Namespaced resource - build module path
                let module_path = entry.namespace_path
                    .iter()
                    .map(|p| sanitize_identifier(p))
                    .collect::<Vec<_>>()
                    .join("::");
                let _ = writeln!(
                    code,
                    "{}pub use crate::{}::{}::{};",
                    pad,
                    entry.resource_type,
                    module_path,
                    const_name
                );
            }
        }
    }
}

