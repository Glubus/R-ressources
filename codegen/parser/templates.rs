/// Template parsing support
use crate::codegen::types::{ResourceValue, Template, TemplateParameter, TemplateParameterType};

/// Tracks template state during parsing
pub struct TemplateState {
    /// Template string with placeholders
    pub template_string: Option<String>,
    /// Collected parameters
    pub parameters: Vec<TemplateParameter>,
}

impl TemplateState {
    pub fn new() -> Self {
        Self {
            template_string: None,
            parameters: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.template_string = None;
        self.parameters.clear();
    }
}

impl Default for TemplateState {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles a template attribute on a string tag
pub fn handle_template_attribute(
    attr_value: &str,
    state: &mut TemplateState,
) {
    state.template_string = Some(attr_value.to_string());
}

/// Handles a <param> tag within a template
pub fn handle_param_tag(
    name: &str,
    type_str: &str,
    state: &mut TemplateState,
) {
    let param_type = match type_str.to_lowercase().as_str() {
        "string" => TemplateParameterType::String,
        "int" => TemplateParameterType::Int,
        "float" => TemplateParameterType::Float,
        "bool" => TemplateParameterType::Bool,
        _ => TemplateParameterType::String, // Default to string
    };

    state.parameters.push(TemplateParameter {
        name: name.to_string(),
        param_type,
    });
}

/// Finalizes a template and creates a ResourceValue
pub fn finalize_template(
    state: &TemplateState,
) -> Option<ResourceValue> {
    if let Some(ref template_str) = state.template_string {
        // Create template even if no parameters (empty template)
        // But we require at least the template string to be present
        if !template_str.is_empty() {
            return Some(ResourceValue::Template(Template {
                template: template_str.clone(),
                parameters: state.parameters.clone(),
            }));
        }
    }
    None
}

