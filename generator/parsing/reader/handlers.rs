use quick_xml::events::{BytesEnd, BytesStart, BytesText};

use crate::generator::parsing::ast::ParsedResource;

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

    // Capture type attribute for numbers
    if matches!(tag.as_str(), "number" | "int" | "float") {
        state.current_number_type = attr_value(e, b"type");
    } else {
        state.current_number_type = None;
    }

    if let Some(name_attr) = attr_value(e, b"name") {
        if state.namespace_stack.is_empty() {
            state.current_name = Some(name_attr);
        } else {
            let mut path = state.namespace_stack.join("/");
            path.push('/');
            path.push_str(&name_attr);
            state.current_name = Some(path);
        }
    }
}

pub(super) fn handle_text(
    state: &ParseState,
    text: &BytesText<'_>,
) -> Option<ParsedResource> {
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
            _ => {}
        }
    }
    None
}

pub(super) fn handle_end(state: &mut ParseState, e: &BytesEnd<'_>) {
    let tag = to_string(e.name().as_ref());

    if tag == "ns" {
        state.namespace_stack.pop();
    }

    if matches!(
        tag.as_str(),
        "string" | "number" | "int" | "float" | "bool" | "color"
    ) {
        state.current_name = None;
    }
    if matches!(tag.as_str(), "number" | "int" | "float") {
        state.current_number_type = None;
    }
    state.current_tag.clear();
}
