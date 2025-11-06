/// Multi-file resource loading
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::environment;
use super::parser;
use super::types::ResourceValue;

/// Scans the res/ directory and loads all XML resource files
pub fn load_all_resources(res_dir: &Path) -> Result<HashMap<String, Vec<(String, ResourceValue)>>, String> {
    if !res_dir.exists() {
        return Err(format!("Resource directory {} does not exist", res_dir.display()));
    }

    let xml_files = find_xml_files(res_dir)?;

    if xml_files.is_empty() {
        return Err("No XML files found in res/ directory".to_string());
    }

    // Parse all XML files
    let mut all_resources: HashMap<String, Vec<(String, ResourceValue)>> = HashMap::new();
    // Track duplicates across files (per type + name)
    let mut seen: HashMap<String, std::collections::HashMap<String, String>> = HashMap::new(); // type -> (name -> file)
    let mut parse_errors = Vec::new();

    for file_path in &xml_files {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read {}: {e}", file_path.display()))?;
        // Preprocess XML by profile before parsing
        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        let filtered = environment::preprocess_xml(&content, &profile);

        match parser::parse_resources(&filtered) {
            Ok(resources) => {
                // Merge resources from this file
                let file_stem = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                for (res_type, items) in resources {
                    let entry = all_resources.entry(res_type.clone()).or_default();

                    // Duplicate detection per type/name across files
                    let seen_for_type = seen.entry(res_type.clone()).or_default();
                    for (name, value) in &items {
                        if let Some(prev_file) = seen_for_type.get(name) {
                            return Err(format!(
                                "Duplicate resource detected: type='{res_type}' name='{name}' in files '{prev_file}' and '{file_stem}'"
                            ));
                        }
                        seen_for_type.insert(name.clone(), file_stem.clone());
                        entry.push((name.clone(), value.clone()));
                    }
                }
            }
            Err(e) => {
                parse_errors.push(format!("Failed to parse {}: {e}", file_path.display()));
            }
        }
    }

    // Report all parsing errors and fail if any occurred
    if !parse_errors.is_empty() {
        return Err(format!(
            "Parsing errors encountered:\n  {}",
            parse_errors.join("\n  ")
        ));
    }

    if all_resources.is_empty() {
        return Err("No resources were successfully parsed".to_string());
    }

    Ok(all_resources)
}

/// Finds all XML files in a directory (non-recursive)
fn find_xml_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut xml_files = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {e}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "xml" {
                    xml_files.push(path);
                }
            }
        }
    }

    xml_files.sort();
    Ok(xml_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_xml_files_nonexistent() {
        let result = find_xml_files(Path::new("nonexistent"));
        assert!(result.is_err());
    }
}

