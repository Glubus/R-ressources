/// Reference resolution for resources
use std::collections::HashMap;

use super::types::ResourceValue;
use super::utils::sanitize_identifier;

/// Resolves a reference to its target resource path
///
/// Example: Reference { `resource_type`: "string", key: "`app_name`" }
/// Returns: "`string::APP_NAME`" or "`crate::string::APP_NAME`"
pub fn resolve_reference_path(resource_type: &str, key: &str, use_crate_prefix: bool) -> String {
    // key may be a path like "ns1/ns2/name"
    let mut parts: Vec<&str> = key.split('/').filter(|s| !s.is_empty()).collect();
    let const_name = parts
        .pop()
        .map(|s| sanitize_identifier(s).to_uppercase())
        .unwrap_or_default();
    let module_path = parts
        .into_iter()
        .map(sanitize_identifier)
        .collect::<Vec<String>>()
        .join("::");

    let type_prefix = if use_crate_prefix {
        format!("crate::{resource_type}")
    } else {
        resource_type.to_string()
    };

    if module_path.is_empty() {
        format!("{type_prefix}::{const_name}")
    } else {
        format!("{type_prefix}::{module_path}::{const_name}")
    }
}

/// Validates that all references point to existing resources
pub fn validate_references(
    resources: &HashMap<String, Vec<(String, ResourceValue)>>,
) -> Vec<String> {
    let mut errors = Vec::new();

    fn validate_single_reference(
        resource_type: &str,
        key: &str,
        res_type: &str,
        name: &str,
        resources: &HashMap<String, Vec<(String, ResourceValue)>>,
        errors: &mut Vec<String>,
    ) {
        // Check if the referenced resource exists
        if let Some(target_resources) = resources.get(resource_type) {
            let key_exists = target_resources.iter().any(|(n, _)| n == key);
            if !key_exists {
                errors.push(format!(
                    "Unresolved reference in {res_type}.{name}: @{resource_type}/{key} does not exist"
                ));
            }
        } else {
            errors.push(format!(
                "Invalid reference in {res_type}.{name}: resource type '{resource_type}' does not exist"
            ));
        }
    }

    for (res_type, items) in resources {
        for (name, value) in items {
            match value {
                ResourceValue::Reference { resource_type, key } => {
                    validate_single_reference(resource_type, key, res_type, name, resources, &mut errors);
                }
                ResourceValue::InterpolatedString(ref parts) => {
                    for part in parts {
                        if let super::types::InterpolationPart::Reference { resource_type, key } = part {
                            validate_single_reference(resource_type, key, res_type, name, resources, &mut errors);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_reference_path() {
        assert_eq!(
            resolve_reference_path("string", "app_name", false),
            "string::APP_NAME"
        );
        assert_eq!(
            resolve_reference_path("color", "primary", true),
            "crate::color::PRIMARY"
        );
    }
}

