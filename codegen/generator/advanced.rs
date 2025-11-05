/// Code generation for advanced types (color, url, dimension)
use crate::codegen::references;
use crate::codegen::types::ResourceValue;
use crate::codegen::utils::sanitize_identifier;
use std::fmt::Write as _;

/// Generates the color module
pub fn generate_color_module(colors: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod color {\n");

    for (name, value) in colors {
        let const_name = sanitize_identifier(name).to_uppercase();
        
        match value {
            ResourceValue::Color(c) => {
                let _ = writeln!(
                    code,
                    "    pub const {}: &str = \"{}\";",
                    const_name,
                    c.escape_debug()
                );
            }
            ResourceValue::Reference { resource_type, key } => {
                // Generate a reference to another resource
                let target = references::resolve_reference_path(resource_type, key, true);
                let _ = writeln!(code, "    pub const {const_name}: &str = {target};");
            }
            _ => {}
        }
    }

    code.push_str("}\n");
    // also generate typed module alongside
    let mut typed = String::from("\npub mod color_t {\n    use crate::Color;\n");
    for (name, value) in colors {
        if let ResourceValue::Color(hex) = value {
            if let Some((a, r, g, b)) = parse_hex_argb(hex) {
                let _ = writeln!(
                    typed,
                    "    pub const {}: Color = Color::new({r}, {g}, {b}, {a});",
                    sanitize_identifier(name).to_uppercase()
                );
            }
        } else if let ResourceValue::Reference { resource_type, key } = value {
            // reference: emit typed const by referencing string then parsing at build-time (same as above)
            // We cannot compute typed const from reference easily; skip for now to keep build-time consts
            let _ = resource_type; let _ = key;
        }
    }
    typed.push_str("}\n");

    code + &typed
}

/// Generates the url module
pub fn generate_url_module(urls: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod url {\n");

    for (name, value) in urls {
        if let ResourceValue::Url(u) = value {
            let _ = writeln!(
                code,
                "    pub const {}: &str = \"{}\";",
                sanitize_identifier(name).to_uppercase(),
                u.escape_debug()
            );
        }
    }

    code.push_str("}\n");
    // typed module
    let mut typed = String::from("\npub mod url_t {\n    use crate::UrlParts;\n");
    for (name, value) in urls {
        if let ResourceValue::Url(u) = value {
            if let Some((scheme, host, path)) = split_url_parts(u) {
                let _ = writeln!(
                    typed,
                    "    pub const {}: UrlParts = UrlParts::new(\"{}\", \"{}\", \"{}\");",
                    sanitize_identifier(name).to_uppercase(),
                    scheme, host, path
                );
            }
        }
    }
    typed.push_str("}\n");

    code + &typed
}

/// Generates the dimension module
pub fn generate_dimension_module(dimensions: &[(String, ResourceValue)]) -> String {
    let mut code = String::from("\npub mod dimension {\n");

    for (name, value) in dimensions {
        if let ResourceValue::Dimension(d) = value {
            let _ = writeln!(
                code,
                "    pub const {}: &str = \"{}\";",
                sanitize_identifier(name).to_uppercase(),
                d.escape_debug()
            );
        }
    }

    code.push_str("}\n");
    code
}

#[allow(clippy::many_single_char_names)]
fn parse_hex_argb(s: &str) -> Option<(u8, u8, u8, u8)> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((0xFF, r, g, b))
        }
        8 => {
            let a = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let r = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let g = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let b = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some((a, r, g, b))
        }
        _ => None,
    }
}

fn split_url_parts(s: &str) -> Option<(String, String, String)> {
    let (scheme, rest) = s.split_once("://")?;
    if let Some((host, path)) = rest.split_once('/') {
        Some((scheme.to_string(), host.to_string(), format!("/{path}")))
    } else {
        Some((scheme.to_string(), rest.to_string(), "/".to_string()))
    }
}


