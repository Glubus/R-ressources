use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

use super::types::ResourceValue;
pub mod basic;
pub mod advanced;

pub mod arrays;
pub mod templates;

/// Parser state for tracking XML parsing context
struct ParserState {
    current_tag: String,
    current_name: String,
    current_profile: Option<String>,
    array_items: Vec<String>,
    in_array: bool,
    namespace_stack: Vec<String>,
    template_state: templates::TemplateState,
    in_template: bool,
}

impl ParserState {
    fn new() -> Self {
        Self {
            current_tag: String::new(),
            current_name: String::new(),
            current_profile: None,
            array_items: Vec::new(),
            in_array: false,
            namespace_stack: Vec::new(),
            template_state: templates::TemplateState::new(),
            in_template: false,
        }
    }
}

/// Parses an XML resources file and extracts all resource definitions.
pub fn parse_resources(xml: &str) -> Result<HashMap<String, Vec<(String, ResourceValue)>>, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut resources: HashMap<String, Vec<(String, ResourceValue)>> = HashMap::new();
    let mut buf = Vec::new();
    let mut state = ParserState::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => handle_start_event(&e, &mut state, &mut resources),
            Ok(Event::Empty(e)) => handle_empty_event(&e, &mut state),
            Ok(Event::Text(e)) => handle_text_event(&e, &mut state, &mut resources),
            Ok(Event::End(e)) => handle_end_event(&e, &mut state, &mut resources),
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parsing error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(resources)
}

/// Handles XML Start events
fn handle_start_event(
    e: &quick_xml::events::BytesStart,
    state: &mut ParserState,
    _resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
    state.current_tag.clone_from(&tag_name);

    // Extract attributes
    let mut name_attr: Option<String> = None;
    let mut template_attr: Option<String> = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => {
                name_attr = Some(String::from_utf8_lossy(&attr.value).to_string());
            }
            b"profile" => {
                state.current_profile = Some(String::from_utf8_lossy(&attr.value).to_string());
            }
            b"template" => {
                template_attr = Some(String::from_utf8_lossy(&attr.value).to_string());
            }
            _ => {}
        }
    }

    // Namespace handling: <ns name="..."> pushes a namespace level
    if tag_name == "ns" {
        if let Some(ns_name) = name_attr {
            state.namespace_stack.push(ns_name);
        }
        state.current_name.clear();
    } else if let Some(local_name) = name_attr {
        // Qualify resource name with namespace path if present
        if state.namespace_stack.is_empty() {
            state.current_name = local_name;
        } else {
            state.current_name = format!("{}/{}", state.namespace_stack.join("/"), local_name);
        }
    }

    // Detect array types
    if tag_name.ends_with("-array") {
        state.in_array = true;
        state.array_items.clear();
    }

    // Handle template attribute on string tags
    if tag_name == "string" {
        if let Some(ref template_str) = template_attr {
            state.in_template = true;
            templates::handle_template_attribute(template_str, &mut state.template_state);
        }
    }

    // Handle <param> tags within templates (non-empty tags)
    if tag_name == "param" && state.in_template {
        parse_param_attributes(e, &mut state.template_state);
    }
}

/// Handles XML Empty events (self-closing tags like <param/>)
/// Note: In quick_xml, Empty events use BytesStart, not a separate type
fn handle_empty_event(
    e: &quick_xml::events::BytesStart,
    state: &mut ParserState,
) {
    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
    if tag_name == "param" && state.in_template {
        parse_param_attributes(e, &mut state.template_state);
    }
}

/// Parses attributes from a param tag (works for both Start and Empty events)
fn parse_param_attributes(
    e: &quick_xml::events::BytesStart,
    template_state: &mut templates::TemplateState,
) {
    let mut param_name: Option<String> = None;
    let mut param_type: Option<String> = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => {
                param_name = Some(String::from_utf8_lossy(&attr.value).to_string());
            }
            b"type" => {
                param_type = Some(String::from_utf8_lossy(&attr.value).to_string());
            }
            _ => {}
        }
    }
    if let (Some(name), Some(type_str)) = (param_name, param_type) {
        templates::handle_param_tag(&name, &type_str, template_state);
    }
}

/// Handles XML Text events
fn handle_text_event(
    e: &quick_xml::events::BytesText,
    state: &mut ParserState,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    let text = String::from_utf8_lossy(e).trim().to_string();
    if text.is_empty() {
        return;
    }

    if state.in_array && state.current_tag == "item" {
        state.array_items.push(text);
    } else if !state.current_name.is_empty() && !state.in_template {
        // Don't process text content for templates (they use the template attribute)
        match state.current_tag.as_str() {
            "string" => basic::handle_string(&text, &state.current_name, resources),
            "int" => basic::handle_int(&text, &state.current_name, resources),
            "float" => basic::handle_float(&text, &state.current_name, resources),
            "bool" => basic::handle_bool(&text, &state.current_name, resources),
            "color" => advanced::handle_color(&text, &state.current_name, resources),
            "url" => advanced::handle_url(&text, &state.current_name, resources),
            "dimension" => advanced::handle_dimension(&text, &state.current_name, resources),
            _ => {}
        }
    }
}

/// Handles XML End events
fn handle_end_event(
    e: &quick_xml::events::BytesEnd,
    state: &mut ParserState,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

    if tag_name.ends_with("-array") && state.in_array {
        arrays::handle_array_end(
            &tag_name,
            &state.current_name,
            &state.array_items,
            resources,
        );
        state.in_array = false;
        state.array_items.clear();
    }

    // Handle template finalization on </string>
    if tag_name == "string" && state.in_template {
        if let Some(template_value) = templates::finalize_template(&state.template_state) {
            resources
                .entry("string".to_string())
                .or_default()
                .push((state.current_name.clone(), template_value));
        }
        state.template_state.reset();
        state.in_template = false;
    }

    // Pop namespace level on </ns>
    if tag_name == "ns" {
        state.namespace_stack.pop();
    }

    state.current_tag.clear();
}


