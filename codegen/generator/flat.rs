/// Code generation for flat `r::` access module
use std::collections::HashMap;

use crate::codegen::types::ResourceValue;
use crate::codegen::utils::sanitize_identifier;
use std::fmt::Write as _;

/// Represents a resource entry for flat module generation
struct ResourceEntry {
    resource_type: String,
    full_path: String,
    module_path: String,
    leaf_name: String,
}

/// Generates a flat module `r` with all resources accessible directly
pub fn generate_r_module(resources: &HashMap<String, Vec<(String, ResourceValue)>>) -> String {
    let mut code = String::from("\n/// Flat access to all resources via `r::RESOURCE_NAME`\npub mod r {\n");

    // Collect all resources and compute minimal unique aliases
    let all_entries = collect_all_resources(resources);
    let aliases = compute_minimal_aliases(&all_entries);
    
    // Generate exports with minimal aliases
    for entry in &all_entries {
        if let Some(alias) = aliases.get(&entry.full_path) {
            if entry.module_path.is_empty() {
                let _ = writeln!(
                    code,
                    "    pub use crate::{}::{} as {};",
                    entry.resource_type,
                    sanitize_identifier(&entry.leaf_name).to_uppercase(),
                    alias
                );
            } else {
                let _ = writeln!(
                    code,
                    "    pub use crate::{}::{}::{} as {};",
                    entry.resource_type,
                    entry.module_path,
                    sanitize_identifier(&entry.leaf_name).to_uppercase(),
                    alias
                );
            }
        }
    }

    code.push_str("}\n");
    
    // typed flat module
    let mut typed = String::from("\n/// Flat access for typed resources via `r_t::RESOURCE_NAME`\npub mod r_t {\n");
    let typed_entries = collect_typed_resources(resources);
    let typed_aliases = compute_minimal_aliases(&typed_entries);
    
    for entry in &typed_entries {
        if let Some(alias) = typed_aliases.get(&entry.full_path) {
            if entry.module_path.is_empty() {
                let _ = writeln!(
                    typed,
                    "    pub use crate::{}::{} as {};",
                    entry.resource_type,
                    sanitize_identifier(&entry.leaf_name).to_uppercase(),
                    alias
                );
            } else {
                let _ = writeln!(
                    typed,
                    "    pub use crate::{}::{}::{} as {};",
                    entry.resource_type,
                    entry.module_path,
                    sanitize_identifier(&entry.leaf_name).to_uppercase(),
                    alias
                );
            }
        }
    }
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
                
                let (module_path, leaf_name) = split_path(name);
                entries.push(ResourceEntry {
                    resource_type: resource_type.to_string(),
                    full_path: name.clone(),
                    module_path,
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
                
                let (module_path, leaf_name) = split_path(name);
                entries.push(ResourceEntry {
                    resource_type: resource_type.to_string(),
                    full_path: name.clone(),
                    module_path,
                    leaf_name: leaf_name.to_string(),
                });
            }
        }
    }
    
    entries
}

/// Splits a path like "auth/errors/invalid_credentials" into (module_path, leaf_name)
fn split_path(path: &str) -> (String, &str) {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return (String::new(), "");
    }
    
    if parts.len() == 1 {
        return (String::new(), parts[0]);
    }
    
    let leaf = parts[parts.len() - 1];
    let module_parts: Vec<String> = parts[..parts.len() - 1]
        .iter()
        .map(|p| sanitize_identifier(p))
        .collect();
    let module_path = module_parts.join("::");
    
    (module_path, leaf)
}

/// Computes minimal unique aliases for all resources
/// Strategy: Start with just the leaf name, if conflict add parent segments progressively
fn compute_minimal_aliases(entries: &[ResourceEntry]) -> HashMap<String, String> {
    use std::collections::HashMap as Map;
    
    let mut aliases: Map<String, String> = Map::new();
    let mut name_to_paths: Map<String, Vec<&str>> = Map::new();
    
    // Group entries by leaf name
    for entry in entries {
        let leaf_upper = sanitize_identifier(&entry.leaf_name).to_uppercase();
        name_to_paths.entry(leaf_upper).or_default().push(&entry.full_path);
    }
    
    // For each leaf name, compute minimal alias
    for (leaf_upper, paths) in name_to_paths {
        if paths.len() == 1 {
            // Unique name - use just the leaf
            aliases.insert(paths[0].to_string(), leaf_upper.clone());
        } else {
            // Conflict - need to prefix progressively
            for path in paths {
                let alias = compute_minimal_prefix(path, &leaf_upper, entries);
                aliases.insert(path.to_string(), alias);
            }
        }
    }
    
    aliases
}

/// Computes the minimal prefix needed to make an alias unique
fn compute_minimal_prefix(
    target_path: &str,
    _leaf_name: &str,
    all_entries: &[ResourceEntry],
) -> String {
    let parts: Vec<&str> = target_path.split('/').filter(|s| !s.is_empty()).collect();
    
    // Try progressively longer prefixes: leaf, parent+leaf, grandparent+parent+leaf, etc.
    for prefix_len in 0..parts.len() {
        let prefix_parts: Vec<&str> = parts[prefix_len..].to_vec();
        let candidate = prefix_parts
            .iter()
            .map(|p| sanitize_identifier(p).to_uppercase())
            .collect::<Vec<_>>()
            .join("_");
        
        // Check if this candidate is unique
        let is_unique = all_entries
            .iter()
            .filter(|e| {
                let e_parts: Vec<&str> = e.full_path.split('/').filter(|s| !s.is_empty()).collect();
                if e_parts.len() <= prefix_len {
                    return false;
                }
                let e_prefix_parts: Vec<&str> = e_parts[prefix_len..].to_vec();
                let e_candidate = e_prefix_parts
                    .iter()
                    .map(|p| sanitize_identifier(p).to_uppercase())
                    .collect::<Vec<_>>()
                    .join("_");
                e_candidate == candidate
            })
            .count()
            == 1;
        
        if is_unique {
            return candidate;
        }
    }
    
    // Fallback: full path (shouldn't happen, but safety)
    sanitize_identifier(target_path).to_uppercase()
}

