//! Système modulaire pour les types de ressources.
//!
//! Pour ajouter un nouveau type, créez simplement un fichier `ir/types/your_type.rs`
//! et implémentez le trait `ResourceType`.
mod bool;
mod color;
mod number;
mod string;

use crate::generator::ir::{
    ResourceKey, ResourceNode, ResourceOrigin,
};
use crate::generator::parsing::ParsedResource;

/// Trait que chaque type de ressource doit implémenter
pub trait ResourceType: Send + Sync {
    /// Nom du type (ex: "string", "number", "bool")
    fn name(&self) -> &'static str;

    /// Tags XML qui correspondent à ce type (ex: ["string"], ["number", "int", "float"])
    #[allow(dead_code)] // Reserved for future use
    fn xml_tags(&self) -> &'static [&'static str];

    /// Returns the ResourceKind corresponding to this type
    fn resource_kind(&self) -> crate::generator::ir::ResourceKind;

    /// Converts a `ParsedResource` into a `ResourceNode`
    fn build_node(
        &self,
        parsed: &ParsedResource,
        origin: ResourceOrigin,
    ) -> Option<ResourceNode>;

    /// Generates Rust code for this type
    fn emit_rust(
        &self,
        key: &ResourceKey,
        node: &ResourceNode,
        indent: usize,
    ) -> Option<String>;
}

/// Registry globale des types de ressources
pub struct TypeRegistry {
    types: Vec<Box<dyn ResourceType>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    /// Registers a new type
    pub fn register(&mut self, ty: Box<dyn ResourceType>) {
        self.types.push(ty);
    }

    /// Finds a type by its name
    pub fn find_by_name(
        &self,
        name: &str,
    ) -> Option<&dyn ResourceType> {
        self.types
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    /// Finds a type by XML tag
    #[allow(dead_code)] // Public API, reserved for future use
    pub fn find_by_xml_tag(
        &self,
        tag: &str,
    ) -> Option<&dyn ResourceType> {
        self.types
            .iter()
            .find(|t| t.xml_tags().contains(&tag))
            .map(|t| t.as_ref())
    }

    /// Returns all registered types
    pub fn all(&self) -> &[Box<dyn ResourceType>] {
        &self.types
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(string::StringType));
        registry.register(Box::new(number::NumberTypeHandler));
        registry.register(Box::new(bool::BoolType));
        registry.register(Box::new(color::ColorType));
        registry
    }
}
