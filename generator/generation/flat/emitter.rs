//! Code emission for flat module generation

use crate::generator::analysis::AnalysisWarning;
use crate::generator::ir::{ResourceGraph, ResourceKey, ResourceNode, TypeRegistry};
use crate::generator::utils::sanitize_identifier;
use std::collections::HashMap;
use std::fmt::Write as _;

use super::tree::{build_namespace_tree, sort_namespace_tree, NamespaceNode};

/// Context for code generation
struct GenerationContext<'a> {
    graph: &'a ResourceGraph,
    registry: &'a TypeRegistry,
    duplicate_info: &'a HashMap<ResourceKey, String>,
}

/// Parameters for emitting a single resource
struct ResourceEmitParams<'a> {
    key: &'a ResourceKey,
    node: &'a ResourceNode,
    warning_message: Option<&'a String>,
    indent: usize,
}

/// Generates the `r` module with nested namespace structure
pub fn generate_r_module(
    graph: &ResourceGraph,
    registry: &TypeRegistry,
    warnings: &[AnalysisWarning],
) -> String {
    let mut tree = build_namespace_tree(graph);
    sort_namespace_tree(&mut tree);

    // Build a map of keys to their warning messages (for duplicate info)
    let duplicate_info: HashMap<_, _> = warnings
        .iter()
        .filter_map(|w| {
            w.key.as_ref().map(|k| (k.clone(), w.message.clone()))
        })
        .collect();

    let ctx = GenerationContext {
        graph,
        registry,
        duplicate_info: &duplicate_info,
    };

    let mut code = String::from("\npub mod r {\n");
    emit_namespace_tree(&mut code, &tree, &ctx, 4);
    code.push_str("}\n");
    code
}

fn emit_namespace_tree(
    code: &mut String,
    node: &NamespaceNode,
    ctx: &GenerationContext<'_>,
    indent: usize,
) {
    let pad = " ".repeat(indent);
    for (ns_name, child) in &node.children {
        let _ = writeln!(
            code,
            "{}pub mod {} {{",
            pad,
            sanitize_identifier(ns_name)
        );
        emit_namespace_tree(code, child, ctx, indent + 4);
        let _ = writeln!(code, "{}}}", pad);
    }

    for key in &node.resource_keys {
        if let Some(all_nodes) = ctx.graph.get_all(key) {
            let warning_message = ctx.duplicate_info.get(key);

            // Only emit the first node (priority), duplicates are ignored but warned
            if let Some(first_node) = all_nodes.first() {
                let params = ResourceEmitParams {
                    key,
                    node: first_node,
                    warning_message,
                    indent,
                };
                emit_resource(code, &params, ctx);
            }
            // Note: Duplicate nodes are not generated, only the first one is kept
        }
    }
}

fn emit_resource(
    code: &mut String,
    params: &ResourceEmitParams<'_>,
    ctx: &GenerationContext<'_>,
) {
    let pad = " ".repeat(params.indent);

    // Add warning annotation for duplicates with file information
    if let Some(warning) = params.warning_message {
        // Extract just the relevant part of the warning for the note
        let note = if warning.len() > 100 {
            format!("{}...", &warning[..100])
        } else {
            warning.clone()
        };
        code.push_str(&format!(
            "{pad}#[deprecated(note = \"{}\")]\n",
            note.replace('"', "\\\"")
        ));
        // Add allow for dead_code with a message
        code.push_str(&format!(
            "{pad}#[allow(dead_code)] // WARNING: Duplicate resource - only first definition is used\n"
        ));
    }

    // Find the type handler by matching ResourceKind
    for ty in ctx.registry.all() {
        if ty.resource_kind() == params.node.kind {
            if let Some(rust_code) = ty.emit_rust(params.key, params.node, params.indent) {
                code.push_str(&rust_code);
            }
            return;
        }
    }
}

