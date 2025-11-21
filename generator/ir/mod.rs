//! Intermediate Representation (IR) for resource graph.
//!
//! This layer receives the AST from parsing and builds a normalized resource graph
//! before analysis and code generation. The `ResourceGraph` is a unified representation
//! that stores all resources with their metadata (origin, profile, namespace).

mod builder;
mod model;
pub mod types;

pub use builder::ResourceGraphBuilder;
pub use model::{
    ResourceGraph, ResourceKey, ResourceKind, ResourceNode,
    ResourceOrigin, ResourceValue,
};
pub use types::TypeRegistry;

// Re-export commonly used types from model (for advanced usage)
#[allow(unused_imports)] // Public API, may be used by consumers
pub use model::{NumberType, NumberValue};
