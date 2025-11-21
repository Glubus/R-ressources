//! Namespace tree construction and sorting

use crate::generator::ir::{ResourceGraph, ResourceKey};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct NamespaceNode {
    pub(super) children: BTreeMap<String, NamespaceNode>,
    pub(super) resource_keys: Vec<ResourceKey>,
}

/// Builds a namespace tree from the resource graph
pub(super) fn build_namespace_tree(graph: &ResourceGraph) -> NamespaceNode {
    let mut root = NamespaceNode::default();
    // Only use the first node for each key (primary), duplicates are handled separately
    for key in graph.nodes().keys() {
        let mut current = &mut root;
        for ns_part in &key.namespace {
            current = current
                .children
                .entry(ns_part.clone())
                .or_default();
        }
        current.resource_keys.push(key.clone());
    }
    root
}

/// Sorts the namespace tree recursively
pub(super) fn sort_namespace_tree(node: &mut NamespaceNode) {
    node.resource_keys.sort_by(|a, b| a.name.cmp(&b.name));
    for child in node.children.values_mut() {
        sort_namespace_tree(child);
    }
}

