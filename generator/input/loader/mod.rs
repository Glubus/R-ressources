mod error;
mod profile;
mod raw_file;
mod scan;

pub use error::LoaderError;
pub use raw_file::RawResourceFile;

use std::fs;
use std::path::Path;

use super::BuildPlan;
use scan::collect_xml_files;

/// Loads every XML file defined in the build plan, applying profile preprocessing.
pub fn load_resources(
    plan: &BuildPlan,
) -> Result<Vec<RawResourceFile>, LoaderError> {
    let mut files = load_directory(
        &plan.resources_dir,
        false,
        &plan.profile,
        true, /* strict */
    )?;

    if let Some(tests_dir) = &plan.tests_resources_dir {
        if tests_dir.exists() {
            let mut test_files = load_directory(
                tests_dir,
                true,
                &plan.profile,
                false, /* not strict */
            )?;
            files.append(&mut test_files);
        }
    }

    Ok(files)
}

fn load_directory(
    dir: &Path,
    is_test: bool,
    profile: &str,
    strict: bool,
) -> Result<Vec<RawResourceFile>, LoaderError> {
    if !dir.exists() {
        if strict {
            return Err(LoaderError::MissingDirectory(
                dir.to_path_buf(),
            ));
        }
        return Ok(Vec::new());
    }

    let xml_paths = collect_xml_files(dir)?;
    if xml_paths.is_empty() {
        if strict {
            return Err(LoaderError::NoXmlFilesFound {
                searched: dir.to_path_buf(),
            });
        }
        return Ok(Vec::new());
    }

    let mut loaded = Vec::with_capacity(xml_paths.len());
    for path in xml_paths {
        let raw = fs::read_to_string(&path).map_err(|source| {
            LoaderError::Io {
                path: path.clone(),
                source,
            }
        })?;
        let filtered = profile::preprocess_xml(&raw, profile);
        loaded.push(RawResourceFile::new(path, filtered, is_test));
    }

    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn fails_when_resources_dir_missing() {
        let tmp = tempdir().unwrap();
        let plan =
            BuildPlan::new(tmp.path().join("res"), None, "debug");

        let err = load_resources(&plan).err().unwrap();
        assert!(matches!(err, LoaderError::MissingDirectory(_)));
    }

    #[test]
    fn load_files_and_apply_profile_filtering() {
        let tmp = tempdir().unwrap();
        let res_dir = tmp.path().join("res");
        fs::create_dir_all(&res_dir).unwrap();

        write_file(
            &res_dir.join("values.xml"),
            r#"
<resources>
    <string name="visible" profile="release">prod</string>
    <string name="hidden" profile="debug">dbg</string>
</resources>
"#,
        );

        let tests_dir = tmp.path().join("res_tests");
        fs::create_dir_all(&tests_dir).unwrap();
        write_file(
            &tests_dir.join("tests.xml"),
            r#"
<resources>
    <string name="test_only">value</string>
</resources>
"#,
        );

        let plan =
            BuildPlan::new(res_dir, Some(tests_dir), "release");
        let files = load_resources(&plan).expect("loader succeeds");

        assert_eq!(files.len(), 2);
        let prod_file = files
            .iter()
            .find(|f| !f.is_test)
            .expect("prod file exists");
        assert!(prod_file.contents.contains("visible"));
        assert!(
            !prod_file.contents.contains("hidden"),
            "debug-only resource should be filtered out"
        );

        let test_file = files
            .iter()
            .find(|f| f.is_test)
            .expect("test file exists");
        assert!(test_file.contents.contains("test_only"));
    }

    #[test]
    fn missing_tests_dir_is_ignored() {
        let tmp = tempdir().unwrap();
        let res_dir = tmp.path().join("res");
        fs::create_dir_all(&res_dir).unwrap();
        write_file(&res_dir.join("values.xml"), "<resources/>");

        let plan = BuildPlan::new(
            res_dir,
            Some(tmp.path().join("missing_tests")),
            "debug",
        );
        let files = load_resources(&plan).expect("loader succeeds");
        assert_eq!(files.len(), 1);
        assert!(!files[0].is_test);
    }
}
