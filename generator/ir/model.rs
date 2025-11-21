use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ResourceGraph {
    nodes: BTreeMap<ResourceKey, Vec<ResourceNode>>, // Multiple nodes per key to track duplicates
}

impl ResourceGraph {
    /// Insert a node. Returns true if this is a duplicate (key already exists)
    pub fn insert(
        &mut self,
        key: ResourceKey,
        node: ResourceNode,
    ) -> bool {
        let is_duplicate = self.nodes.contains_key(&key);
        self.nodes.entry(key).or_default().push(node);
        is_duplicate
    }

    pub fn nodes(&self) -> &BTreeMap<ResourceKey, Vec<ResourceNode>> {
        &self.nodes
    }

    /// Get the first (primary) node for a key
    #[allow(dead_code)] // Used in tests
    pub fn get(&self, key: &ResourceKey) -> Option<&ResourceNode> {
        self.nodes.get(key).and_then(|nodes| nodes.first())
    }

    /// Get all nodes for a key (including duplicates)
    pub fn get_all(
        &self,
        key: &ResourceKey,
    ) -> Option<&[ResourceNode]> {
        self.nodes.get(key).map(|v| v.as_slice())
    }

    /// Check if a key has duplicates
    #[allow(dead_code)] // Reserved for future use
    pub fn has_duplicates(&self, key: &ResourceKey) -> bool {
        self.nodes.get(key).is_some_and(|nodes| nodes.len() > 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKey {
    pub namespace: Vec<String>,
    pub name: String,
}

impl ResourceKey {
    #[allow(dead_code)] // Reserved for future use
    pub fn new(
        namespace: Vec<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            namespace,
            name: name.into(),
        }
    }

    pub fn from_path(path: &str) -> Self {
        let mut parts: Vec<String> = path
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|s| s.to_string())
            .collect();
        let name = parts.pop().unwrap_or_default();
        Self {
            namespace: parts,
            name,
        }
    }

    pub fn full_name(&self) -> String {
        if self.namespace.is_empty() {
            return self.name.clone();
        }
        format!("{}/{}", self.namespace.join("/"), self.name)
    }
}

#[derive(Debug, Clone)]
pub struct ResourceNode {
    pub kind: ResourceKind,
    pub value: ResourceValue,
    pub origin: ResourceOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum ResourceKind {
    String,
    Number,
    Bool,
    Color,
    Url,
    Dimension,
    Array(String),
    Template,
    Custom(String),
}

/// Represents the explicit Rust type requested via `<number type="...">`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl NumberType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

/// Represents a parsed numeric value
#[derive(Debug, Clone)]
pub enum NumberValue {
    /// Fits into i64
    Int(i64),
    /// Fits into f64
    Float(f64),
    /// Requires arbitrary precision
    BigDecimal(String),
    /// Explicitly typed numeric constant
    Typed { literal: String, ty: NumberType },
}

#[derive(Debug, Clone)]
pub enum ResourceValue {
    String(String),
    Number(NumberValue),
    Bool(bool),
    Color(String),
    // TODO: add arrays, templates, references, etc.
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for future use (line, profile, is_test)
pub struct ResourceOrigin {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub profile: Option<String>,
    pub is_test: bool,
}

impl ResourceOrigin {
    pub fn new(file: PathBuf, is_test: bool) -> Self {
        Self {
            file,
            line: None,
            profile: None,
            is_test,
        }
    }
}
