# Modular Resource Type System

This module allows you to easily add new resource types by simply creating a new file.

## How to Add a New Type

### 1. Create the Type File

Create a file `ir/types/your_type.rs` with a struct that implements the `ResourceType` trait:

```rust
use crate::generator::ir::types::ResourceType;
use crate::generator::ir::{ResourceKey, ResourceKind, ResourceNode, ResourceOrigin, ResourceValue};
use crate::generator::parsing::{ParsedResource, ScalarValue};
use crate::generator::utils::sanitize_identifier;

pub struct YourType;

impl ResourceType for YourType {
    fn name(&self) -> &'static str {
        "your_type"
    }

    fn xml_tags(&self) -> &'static [&'static str] {
        &["your-tag", "other-tag"]  // XML tags that trigger this type
    }

    fn resource_kind(&self) -> ResourceKind {
        ResourceKind::Custom("your_type".to_string())  // Or an existing variant
    }

    fn build_node(
        &self,
        parsed: &ParsedResource,
        origin: ResourceOrigin,
    ) -> Option<ResourceNode> {
        // Convert ParsedResource to ResourceNode
        // Return None if conversion fails
        Some(ResourceNode {
            kind: self.resource_kind(),
            value: ResourceValue::String(/* ... */),
            origin,
        })
    }

    fn emit_rust(&self, key: &ResourceKey, node: &ResourceNode, indent: usize) -> Option<String> {
        // Generate Rust code for this type
        let pad = " ".repeat(indent);
        let const_name = sanitize_identifier(&key.name).to_uppercase();
        Some(format!("{pad}pub const {const_name}: /* type */ = /* value */;\n"))
    }
}
```

### 2. Register the Type

In `ir/types/mod.rs`, add:

```rust
mod your_type;

impl Default for TypeRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // ... existing types ...
        registry.register(Box::new(your_type::YourType));
        registry
    }
}
```

### 3. Add ResourceKind (if new)

If you're using a new `ResourceKind`, add it in `ir/model.rs`:

```rust
pub enum ResourceKind {
    // ... existing variants ...
    YourType,
}
```

### 4. Add ResourceValue (if new)

If you're using a new `ResourceValue`, add it in `ir/model.rs`:

```rust
pub enum ResourceValue {
    // ... existing variants ...
    YourType(String),  // or appropriate structure
}
```

That's it! The system will automatically discover your new type and use it for parsing and code generation.
