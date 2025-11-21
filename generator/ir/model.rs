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
    Template {
        text: String,
        params: Vec<TemplateParam>,
    },
    // TODO: add arrays, references, etc.
}

#[derive(Debug, Clone)]
pub struct TemplateParam {
    pub name: String,
    pub value: TemplateParamValue, // Store parameter type information
}

#[derive(Debug, Clone)]
pub enum TemplateParamValue {
    String,
    Number { explicit_type: Option<String> }, // Store explicit_type for numbers (e.g., "bigdecimal", "i32")
    Bool,
    Color,
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

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for ResourceGraph
    #[test]
    fn graph_default_is_empty() {
        let graph = ResourceGraph::default();
        assert_eq!(graph.nodes().len(), 0);
    }

    #[test]
    fn graph_insert_single_node() {
        let mut graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "test");
        let node = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("value".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let is_duplicate = graph.insert(key.clone(), node);
        assert!(!is_duplicate);
        assert_eq!(graph.nodes().len(), 1);
        assert!(graph.get(&key).is_some());
    }

    #[test]
    fn graph_insert_detects_duplicate() {
        let mut graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "test");
        let node1 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("first".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test1.xml"), false),
        };
        let node2 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("second".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test2.xml"), false),
        };

        let is_dup1 = graph.insert(key.clone(), node1);
        assert!(!is_dup1);

        let is_dup2 = graph.insert(key.clone(), node2);
        assert!(is_dup2);
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn graph_get_returns_first_node() {
        let mut graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "test");
        let node1 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("first".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test1.xml"), false),
        };
        let node2 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("second".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test2.xml"), false),
        };

        graph.insert(key.clone(), node1);
        graph.insert(key.clone(), node2);

        let first = graph.get(&key).expect("node exists");
        match &first.value {
            ResourceValue::String(value) => assert_eq!(value, "first"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn graph_get_all_returns_all_nodes() {
        let mut graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "test");
        let node1 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("first".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test1.xml"), false),
        };
        let node2 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("second".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test2.xml"), false),
        };

        graph.insert(key.clone(), node1);
        graph.insert(key.clone(), node2);

        let all = graph.get_all(&key).expect("nodes exist");
        assert_eq!(all.len(), 2);
        match &all[0].value {
            ResourceValue::String(value) => assert_eq!(value, "first"),
            _ => panic!("expected String"),
        }
        match &all[1].value {
            ResourceValue::String(value) => assert_eq!(value, "second"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn graph_has_duplicates() {
        let mut graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "test");
        let node1 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("first".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test1.xml"), false),
        };
        let node2 = ResourceNode {
            kind: ResourceKind::String,
            value: ResourceValue::String("second".to_string()),
            origin: ResourceOrigin::new(PathBuf::from("test2.xml"), false),
        };

        graph.insert(key.clone(), node1);
        assert!(!graph.has_duplicates(&key));

        graph.insert(key.clone(), node2);
        assert!(graph.has_duplicates(&key));
    }

    #[test]
    fn graph_get_nonexistent_key() {
        let graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "nonexistent");
        assert!(graph.get(&key).is_none());
    }

    #[test]
    fn graph_get_all_nonexistent_key() {
        let graph = ResourceGraph::default();
        let key = ResourceKey::new(vec![], "nonexistent");
        assert!(graph.get_all(&key).is_none());
    }

    // Tests for ResourceKey
    #[test]
    fn resource_key_new() {
        let key = ResourceKey::new(vec!["ns1".to_string(), "ns2".to_string()], "name");
        assert_eq!(key.namespace, vec!["ns1", "ns2"]);
        assert_eq!(key.name, "name");
    }

    #[test]
    fn resource_key_new_with_string() {
        let key = ResourceKey::new(vec![], "name");
        assert_eq!(key.namespace, Vec::<String>::new());
        assert_eq!(key.name, "name");
    }

    #[test]
    fn resource_key_from_path_simple() {
        let key = ResourceKey::from_path("name");
        assert_eq!(key.namespace, Vec::<String>::new());
        assert_eq!(key.name, "name");
    }

    #[test]
    fn resource_key_from_path_with_namespace() {
        let key = ResourceKey::from_path("ns1/ns2/name");
        assert_eq!(key.namespace, vec!["ns1", "ns2"]);
        assert_eq!(key.name, "name");
    }

    #[test]
    fn resource_key_from_path_single_namespace() {
        let key = ResourceKey::from_path("ns/name");
        assert_eq!(key.namespace, vec!["ns"]);
        assert_eq!(key.name, "name");
    }

    #[test]
    fn resource_key_from_path_with_empty_parts() {
        let key = ResourceKey::from_path("//ns1///name//");
        assert_eq!(key.namespace, vec!["ns1"]);
        assert_eq!(key.name, "name");
    }

    #[test]
    fn resource_key_from_path_empty() {
        let key = ResourceKey::from_path("");
        assert_eq!(key.namespace, Vec::<String>::new());
        assert_eq!(key.name, "");
    }

    #[test]
    fn resource_key_from_path_only_slashes() {
        let key = ResourceKey::from_path("///");
        assert_eq!(key.namespace, Vec::<String>::new());
        assert_eq!(key.name, "");
    }

    #[test]
    fn resource_key_full_name_no_namespace() {
        let key = ResourceKey::new(vec![], "name");
        assert_eq!(key.full_name(), "name");
    }

    #[test]
    fn resource_key_full_name_with_namespace() {
        let key = ResourceKey::new(vec!["ns1".to_string(), "ns2".to_string()], "name");
        assert_eq!(key.full_name(), "ns1/ns2/name");
    }

    #[test]
    fn resource_key_full_name_single_namespace() {
        let key = ResourceKey::new(vec!["ns".to_string()], "name");
        assert_eq!(key.full_name(), "ns/name");
    }

    #[test]
    fn resource_key_equality() {
        let key1 = ResourceKey::new(vec!["ns".to_string()], "name");
        let key2 = ResourceKey::new(vec!["ns".to_string()], "name");
        assert_eq!(key1, key2);
    }

    #[test]
    fn resource_key_inequality() {
        let key1 = ResourceKey::new(vec!["ns1".to_string()], "name");
        let key2 = ResourceKey::new(vec!["ns2".to_string()], "name");
        assert_ne!(key1, key2);
    }

    #[test]
    fn resource_key_ordering() {
        let key1 = ResourceKey::new(vec![], "a");
        let key2 = ResourceKey::new(vec![], "b");
        assert!(key1 < key2);
    }

    // Tests for NumberType
    #[test]
    fn number_type_as_str_i8() {
        assert_eq!(NumberType::I8.as_str(), "i8");
    }

    #[test]
    fn number_type_as_str_i16() {
        assert_eq!(NumberType::I16.as_str(), "i16");
    }

    #[test]
    fn number_type_as_str_i32() {
        assert_eq!(NumberType::I32.as_str(), "i32");
    }

    #[test]
    fn number_type_as_str_i64() {
        assert_eq!(NumberType::I64.as_str(), "i64");
    }

    #[test]
    fn number_type_as_str_u8() {
        assert_eq!(NumberType::U8.as_str(), "u8");
    }

    #[test]
    fn number_type_as_str_u16() {
        assert_eq!(NumberType::U16.as_str(), "u16");
    }

    #[test]
    fn number_type_as_str_u32() {
        assert_eq!(NumberType::U32.as_str(), "u32");
    }

    #[test]
    fn number_type_as_str_u64() {
        assert_eq!(NumberType::U64.as_str(), "u64");
    }

    #[test]
    fn number_type_as_str_f32() {
        assert_eq!(NumberType::F32.as_str(), "f32");
    }

    #[test]
    fn number_type_as_str_f64() {
        assert_eq!(NumberType::F64.as_str(), "f64");
    }

    #[test]
    fn number_type_equality() {
        assert_eq!(NumberType::I32, NumberType::I32);
        assert_ne!(NumberType::I32, NumberType::I64);
    }

    // Tests for ResourceOrigin
    #[test]
    fn resource_origin_new() {
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);
        assert_eq!(origin.file, PathBuf::from("test.xml"));
        assert_eq!(origin.line, None);
        assert_eq!(origin.profile, None);
        assert!(!origin.is_test);
    }

    #[test]
    fn resource_origin_new_test() {
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), true);
        assert!(origin.is_test);
    }

    // Tests for ResourceKind
    #[test]
    fn resource_kind_equality() {
        assert_eq!(ResourceKind::String, ResourceKind::String);
        assert_ne!(ResourceKind::String, ResourceKind::Number);
    }

    #[test]
    fn resource_kind_array() {
        let kind = ResourceKind::Array("string".to_string());
        match kind {
            ResourceKind::Array(ty) => assert_eq!(ty, "string"),
            _ => panic!("expected Array"),
        }
    }

    #[test]
    fn resource_kind_custom() {
        let kind = ResourceKind::Custom("my_type".to_string());
        match kind {
            ResourceKind::Custom(ty) => assert_eq!(ty, "my_type"),
            _ => panic!("expected Custom"),
        }
    }

    // Tests for ResourceValue
    #[test]
    fn resource_value_string() {
        let value = ResourceValue::String("test".to_string());
        match value {
            ResourceValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn resource_value_number() {
        let value = ResourceValue::Number(NumberValue::Int(42));
        match value {
            ResourceValue::Number(NumberValue::Int(i)) => assert_eq!(i, 42),
            _ => panic!("expected Number::Int"),
        }
    }

    #[test]
    fn resource_value_bool() {
        let value = ResourceValue::Bool(true);
        match value {
            ResourceValue::Bool(b) => assert!(b),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn resource_value_color() {
        let value = ResourceValue::Color("#FF0000".to_string());
        match value {
            ResourceValue::Color(c) => assert_eq!(c, "#FF0000"),
            _ => panic!("expected Color"),
        }
    }

    // Tests for NumberValue
    #[test]
    fn number_value_int() {
        let value = NumberValue::Int(42);
        match value {
            NumberValue::Int(i) => assert_eq!(i, 42),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn number_value_float() {
        let value = NumberValue::Float(3.14);
        match value {
            NumberValue::Float(f) => assert!((f - 3.14).abs() < 0.0001),
            _ => panic!("expected Float"),
        }
    }

    #[test]
    fn number_value_big_decimal() {
        let value = NumberValue::BigDecimal("123.456".to_string());
        match value {
            NumberValue::BigDecimal(s) => assert_eq!(s, "123.456"),
            _ => panic!("expected BigDecimal"),
        }
    }

    #[test]
    fn number_value_typed() {
        let value = NumberValue::Typed {
            literal: "127".to_string(),
            ty: NumberType::I8,
        };
        match value {
            NumberValue::Typed { literal, ty } => {
                assert_eq!(literal, "127");
                assert_eq!(ty, NumberType::I8);
            }
            _ => panic!("expected Typed"),
        }
    }
}
