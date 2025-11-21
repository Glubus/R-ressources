use quick_xml::events::{BytesEnd, BytesStart, BytesText};

use crate::generator::parsing::ast::{ParsedResource, TemplateParam};

use super::state::ParseState;
use super::utils::{attr_value, text_to_string, to_string};

pub(super) fn handle_start(
    state: &mut ParseState,
    e: &BytesStart<'_>,
) {
    let tag = to_string(e.name().as_ref());
    
    state.current_tag = tag.clone();

    if tag == "ns" {
        if let Some(ns_name) = attr_value(e, b"name") {
            state.namespace_stack.push(ns_name);
        }
        state.current_name = None;
        return;
    }

    // Initialize template state FIRST (before processing parameters)
    if tag == "template" {
        state.in_template = true;
        state.template_params.clear();
        state.template_text.clear();
    }
    
    // Capture type attribute for numbers (reuse existing logic)
    let number_type = if matches!(tag.as_str(), "number" | "int" | "float") {
        attr_value(e, b"type")
    } else {
        None
    };

    let name_attr = attr_value(e, b"name");
    let param_name = if let Some(name) = &name_attr {
        if state.namespace_stack.is_empty() {
            Some(name.clone())
        } else {
            let mut path = state.namespace_stack.join("/");
            path.push('/');
            path.push_str(name);
            Some(path)
        }
    } else {
        None
    };

    // Handle template parameters: if we're inside a template, treat standard tags as parameters
    // Reuse existing parsing logic by creating ScalarValue directly from attributes
    if state.in_template && tag != "template" {
        if let Some(param_name_str) = &param_name {
            let param_value = match tag.as_str() {
                "string" => Some(crate::generator::parsing::ScalarValue::Text(String::new())), // Empty for params
                "number" | "int" | "float" => {
                    Some(crate::generator::parsing::ScalarValue::Number {
                        value: String::new(), // Empty for params
                        explicit_type: number_type.clone(),
                    })
                }
                "bool" => Some(crate::generator::parsing::ScalarValue::Bool(false)), // Dummy value for params
                "color" => Some(crate::generator::parsing::ScalarValue::Color(String::new())), // Empty for params
                _ => None,
            };
            
            if let Some(value) = param_value {
                state.template_params.push(TemplateParam {
                    name: param_name_str.clone(),
                    value,
                });
                // Reset current_tag to "template" so text is captured correctly
                state.current_tag = "template".to_string();
                // Don't set current_name for template parameters
                return;
            }
        }
    }

    // Set state for normal resource processing
    state.current_number_type = number_type;
    state.current_name = param_name;
}

pub(super) fn handle_text(
    state: &mut ParseState,
    text: &BytesText<'_>,
) -> Option<ParsedResource> {
    // If we're inside a template, only accumulate text that's directly inside the template tag
    if state.in_template {
        if state.current_tag == "template" {
            let trimmed = text_to_string(text).trim().to_string();
            if !trimmed.is_empty() {
                state.template_text.push_str(&trimmed);
                state.template_text.push(' ');
            }
        }
        // Don't create resources from parameter tags inside templates (they're already handled in handle_start)
        return None;
    }
    
    if let Some(name) = &state.current_name {
        let trimmed = text_to_string(text).trim().to_string();
        if trimmed.is_empty() {
            return None;
        }

        match state.current_tag.as_str() {
            "string" => {
                return Some(ParsedResource::string(name, trimmed))
            }
            "number" | "int" | "float" => {
                return Some(ParsedResource::number(
                    name,
                    trimmed,
                    state.current_number_type.clone(),
                ));
            }
            "bool" => {
                if let Ok(b) = trimmed.parse::<bool>() {
                    return Some(ParsedResource::bool(name, b));
                }
            }
            "color" => {
                return Some(ParsedResource {
                    name: name.clone(),
                    kind:
                        crate::generator::parsing::ResourceKind::Color,
                    value:
                        crate::generator::parsing::ScalarValue::Color(
                            trimmed,
                        ),
                });
            }
            "template" => {
                // Accumulate text for templates (may be called multiple times)
                state.template_text.push_str(&trimmed);
                state.template_text.push(' ');
            }
            _ => {}
        }
    }
    None
}

pub(super) fn handle_end(
    state: &mut ParseState,
    e: &BytesEnd<'_>,
) -> Option<ParsedResource> {
    let tag = to_string(e.name().as_ref());

    if tag == "ns" {
        state.namespace_stack.pop();
        return None;
    }

    // Finalize template when closing tag is encountered
    if tag == "template" {
        let name = state.current_name.clone();
        if let Some(name) = name {
            let text = state.template_text.trim().to_string();
            let params = state.template_params.clone();
            
            // Reset template state
            state.in_template = false;
            state.template_params.clear();
            state.template_text.clear();
            state.current_name = None;
            
            return Some(ParsedResource {
                name,
                kind: crate::generator::parsing::ResourceKind::Template,
                value: crate::generator::parsing::ScalarValue::Template {
                    text,
                    params,
                },
            });
        }
    }
    
    // When inside a template, don't process closing tags of parameter tags as resources
    // (they're already handled in handle_start)
    if state.in_template {
        if matches!(tag.as_str(), "string" | "number" | "int" | "float" | "bool" | "color") {
            // These are template parameters, not resources - just clear current_name
            state.current_name = None;
            return None;
        }
    }

    if matches!(
        tag.as_str(),
        "string" | "number" | "int" | "float" | "bool" | "color" | "template"
    ) {
        state.current_name = None;
    }
    if matches!(tag.as_str(), "number" | "int" | "float") {
        state.current_number_type = None;
    }
    state.current_tag.clear();
    None
}
