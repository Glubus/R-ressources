use std::path::PathBuf;

/// Raw resource file loaded from disk and preprocessed for the selected profile.
#[derive(Debug, Clone)]
pub struct RawResourceFile {
    pub path: PathBuf,
    pub contents: String,
    pub is_test: bool,
}

impl RawResourceFile {
    pub fn new(
        path: PathBuf,
        contents: String,
        is_test: bool,
    ) -> Self {
        Self {
            path,
            contents,
            is_test,
        }
    }
}
