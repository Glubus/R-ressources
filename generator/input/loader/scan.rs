use std::fs;
use std::path::{Path, PathBuf};

use super::LoaderError;

pub(super) fn collect_xml_files(
    dir: &Path,
) -> Result<Vec<PathBuf>, LoaderError> {
    let mut files = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|source| LoaderError::Io {
            path: dir.to_path_buf(),
            source,
        })?;

    for entry in entries {
        let entry = entry.map_err(|source| LoaderError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "xml") {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}
