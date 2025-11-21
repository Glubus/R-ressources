use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ParsedResourceFile {
    pub path: PathBuf,
    pub is_test: bool,
    pub resources: Vec<ParsedResource>,
}

impl ParsedResourceFile {
    pub fn new(
        path: PathBuf,
        is_test: bool,
        resources: Vec<ParsedResource>,
    ) -> Self {
        Self {
            path,
            is_test,
            resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResource {
    pub name: String,
    pub kind: ResourceKind,
    pub value: ScalarValue,
}

impl ParsedResource {
    pub fn string(
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: ResourceKind::String,
            value: ScalarValue::Text(value.into()),
        }
    }

    pub fn number(
        name: impl Into<String>,
        value: impl Into<String>,
        explicit_type: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: ResourceKind::Number,
            value: ScalarValue::Number {
                value: value.into(),
                explicit_type,
            },
        }
    }

    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            kind: ResourceKind::Bool,
            value: ScalarValue::Bool(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    String,
    Number,
    Bool,
    Color,
    // TODO: array, template, etc.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarValue {
    Text(String),
    Number {
        value: String,
        explicit_type: Option<String>, // For type="i32", etc.
    },
    Bool(bool),
    Color(String),
}

impl ScalarValue {
    #[allow(dead_code)] // Used in tests
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value.as_str()),
            Self::Number { value, .. } => Some(value.as_str()),
            Self::Bool(_) => None,
            Self::Color(_) => None,
        }
    }

    #[allow(dead_code)] // Used in tests
    pub fn as_number(&self) -> Option<&str> {
        match self {
            Self::Number { value, .. } => Some(value.as_str()),
            _ => None,
        }
    }

    #[allow(dead_code)] // Used in tests
    pub fn number_explicit_type(&self) -> Option<&str> {
        match self {
            Self::Number { explicit_type, .. } => {
                explicit_type.as_deref()
            }
            _ => None,
        }
    }

    #[allow(dead_code)] // Used in tests
    pub fn as_color(&self) -> Option<&str> {
        match self {
            Self::Color(value) => Some(value.as_str()),
            _ => None,
        }
    }

    #[allow(dead_code)] // Used in tests
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }
}
