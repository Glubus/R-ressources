//! Input layer: file discovery and preprocessing.
//!
//! This module handles:
//! - Scanning directories for XML resource files
//! - Profile-based preprocessing (filtering resources by build profile)
//! - Test resource discovery (from `res/tests/` directory)
//!
//! The output is a list of `RawResourceFile` objects ready for parsing.

pub mod loader;

pub use loader::{load_resources, LoaderError, RawResourceFile};

pub struct BuildPlan {
    /// Root directory that contains runtime resources (default: `res/`).
    pub resources_dir: std::path::PathBuf,
    /// Optional directory for test-only resources.
    pub tests_resources_dir: Option<std::path::PathBuf>,
    /// Cargo profile (debug/release) captured for preprocessing.
    pub profile: String,
}

impl BuildPlan {
    #[allow(dead_code)] // Used in tests and public API
    pub fn new(
        resources_dir: std::path::PathBuf,
        tests_resources_dir: Option<std::path::PathBuf>,
        profile: impl Into<String>,
    ) -> Self {
        Self {
            resources_dir,
            tests_resources_dir,
            profile: profile.into(),
        }
    }
}
