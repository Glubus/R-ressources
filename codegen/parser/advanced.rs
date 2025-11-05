use std::collections::HashMap;

use crate::codegen::types::ResourceValue;

pub fn handle_color(
    text: &str,
    current_name: &str,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    let value = if text.starts_with('@') {
        ResourceValue::parse_string_value(text)
    } else {
        ResourceValue::Color(text.to_string())
    };
    resources
        .entry("color".to_string())
        .or_default()
        .push((current_name.to_string(), value));
}

#[allow(dead_code)]
pub fn handle_position(
    text: &str,
    _current_tag: &str,
    current_name: &str,
    _array_items: &Vec<String>,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    // We expect to be called from parser walking nested tags; here we only accept when closing
    // As a simple approach, support inline "x,y" fallback inside text for now (v0.4 proper structure in parser/mod.rs)
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    if parts.len() == 2 {
        if let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            resources
                .entry("position".to_string())
                .or_default()
                .push((current_name.to_string(), ResourceValue::Dimension(format!("{x},{y}"))));
        }
    }
}

#[allow(dead_code)]
pub fn handle_latlng(
    text: &str,
    _current_tag: &str,
    current_name: &str,
    _array_items: &Vec<String>,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    if parts.len() == 2 {
        if let (Ok(lat), Ok(lng)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) {
                resources
                    .entry("latlng".to_string())
                    .or_default()
                    .push((current_name.to_string(), ResourceValue::Dimension(format!("{lat},{lng}"))));
            }
        }
    }
}

pub fn handle_url(
    text: &str,
    current_name: &str,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    resources
        .entry("url".to_string())
        .or_default()
        .push((current_name.to_string(), ResourceValue::Url(text.to_string())));
}

pub fn handle_dimension(
    text: &str,
    current_name: &str,
    resources: &mut HashMap<String, Vec<(String, ResourceValue)>>,
) {
    resources
        .entry("dimension".to_string())
        .or_default()
        .push((current_name.to_string(), ResourceValue::Dimension(text.to_string())));
}


