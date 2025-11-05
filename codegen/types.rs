/// Represents a part of an interpolated string
#[derive(Debug, Clone)]
pub enum InterpolationPart {
    /// Literal text
    Text(String),
    /// Reference to another resource
    Reference { resource_type: String, key: String },
}

/// Represents a template parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateParameterType {
    /// String parameter
    String,
    /// Integer parameter (i64)
    Int,
    /// Float parameter (f64)
    Float,
    /// Boolean parameter
    Bool,
}

/// Represents a template parameter definition
#[derive(Debug, Clone)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: TemplateParameterType,
}

/// Represents a template with placeholders
#[derive(Debug, Clone)]
pub struct Template {
    /// Template string with {param} placeholders
    pub template: String,
    /// List of parameters in order
    pub parameters: Vec<TemplateParameter>,
}

/// Represents the different types of resource values that can be parsed from XML
#[derive(Debug, Clone)]
pub enum ResourceValue {
    /// A simple string value
    String(String),
    /// An integer value (i64)
    Int(i64),
    /// A floating-point value (f64)
    Float(f64),
    /// A boolean value
    Bool(bool),
    /// A color value (hex string like #FF5722 or #AAFF5722)
    Color(String),
    /// A URL string  
    Url(String),
    /// A dimension value with unit (e.g., "16dp", "24px", "1.5em")
    Dimension(String),
    /// An array of strings
    StringArray(Vec<String>),
    /// An array of integers
    IntArray(Vec<i64>),
    /// An array of floats
    FloatArray(Vec<f64>),
    /// A reference to another resource (e.g., @`string/app_name`)
    Reference { resource_type: String, key: String },
    /// An interpolated string with embedded references (e.g., "Welcome to @string/app_name!")
    InterpolatedString(Vec<InterpolationPart>),
    /// A template string with parameters (e.g., "Hello {name}, you have {count} messages!")
    Template(Template),
}

impl ResourceValue {
    /// Returns the type name of this resource value as a String
    #[allow(dead_code)]
    pub fn type_name(&self) -> String {
        match self {
            Self::String(_) => "string".to_string(),
            Self::Int(_) => "int".to_string(),
            Self::Float(_) => "float".to_string(),
            Self::Bool(_) => "bool".to_string(),
            Self::Color(_) => "color".to_string(),
            Self::Url(_) => "url".to_string(),
            Self::Dimension(_) => "dimension".to_string(),
            Self::StringArray(_) => "string_array".to_string(),
            Self::IntArray(_) => "int_array".to_string(),
            Self::FloatArray(_) => "float_array".to_string(),
            Self::Reference { resource_type, .. } => resource_type.clone(),
            Self::InterpolatedString(_) => "string".to_string(),
            Self::Template(_) => "string".to_string(),
        }
    }
    
    /// Parses a string and returns a `ResourceValue` (detecting references and interpolations)
    pub fn parse_string_value(s: &str) -> Self {
        // Check if it's a pure reference (entire string is @type/name)
        if s.starts_with('@') && !s.contains(' ') && s.matches('@').count() == 1 {
            // Reference format: @type/name
            if let Some((resource_type, key)) = s[1..].split_once('/') {
                return Self::Reference {
                    resource_type: resource_type.to_string(),
                    key: key.to_string(),
                };
            }
        }
        
        // Check for interpolated strings (contains @type/name pattern)
        if s.contains('@') {
            let mut parts = Vec::new();
            let mut current_pos = 0;
            let bytes = s.as_bytes();
            
            while current_pos < bytes.len() {
                // Look for @ symbol
                if let Some(at_pos) = bytes[current_pos..].iter().position(|&b| b == b'@') {
                    let at_pos = current_pos + at_pos;
                    
                    // Add text before @
                    if at_pos > current_pos {
                        let text = String::from_utf8_lossy(&bytes[current_pos..at_pos]).to_string();
                        if !text.is_empty() {
                            parts.push(InterpolationPart::Text(text));
                        }
                    }
                    
                    // Try to parse reference after @
                    let after_at = &s[at_pos + 1..];
                    if let Some(slash_pos) = after_at.find('/') {
                        let resource_type = &after_at[..slash_pos];
                        let after_slash = &after_at[slash_pos + 1..];
                        // Find the end of the reference: first non-alphanumeric/underscore/slash character
                        let ref_end = after_slash
                            .char_indices()
                            .find(|(_, c)| !c.is_alphanumeric() && *c != '_' && *c != '/')
                            .map(|(i, _)| i)
                            .unwrap_or(after_slash.len());
                        let mut key_slice = &after_slash[..ref_end];
                        // Trim trailing '/' that belongs to following literal or next reference
                        let had_trailing_slash = key_slice.ends_with('/');
                        if had_trailing_slash {
                            key_slice = &key_slice[..key_slice.len().saturating_sub(1)];
                        }
                        
                        parts.push(InterpolationPart::Reference {
                            resource_type: resource_type.to_string(),
                            key: key_slice.to_string(),
                        });
                        // Do not consume the trailing '/' if it was trimmed; let it appear as literal text
                        let consumed = if had_trailing_slash { ref_end.saturating_sub(1) } else { ref_end };
                        current_pos = at_pos + 1 + slash_pos + 1 + consumed;
                    } else {
                        // Invalid @ reference, treat as literal
                        parts.push(InterpolationPart::Text("@".to_string()));
                        current_pos = at_pos + 1;
                    }
                } else {
                    // No more @, add remaining text
                    let text = String::from_utf8_lossy(&bytes[current_pos..]).to_string();
                    if !text.is_empty() {
                        parts.push(InterpolationPart::Text(text));
                    }
                    break;
                }
            }
            
            // If we found interpolation parts, return InterpolatedString
            if !parts.is_empty() {
                return Self::InterpolatedString(parts);
            }
        }
        
        // Otherwise it's a simple string
        Self::String(s.to_string())
    }
}


