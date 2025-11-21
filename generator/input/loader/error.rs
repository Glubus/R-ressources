use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LoaderError {
    MissingDirectory(PathBuf),
    Io { path: PathBuf, source: io::Error },
    NoXmlFilesFound { searched: PathBuf },
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDirectory(path) => {
                write!(
                    f,
                    "directory '{}' does not exist",
                    path.display()
                )
            }
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to read '{}': {source}",
                    path.display()
                )
            }
            Self::NoXmlFilesFound { searched } => {
                write!(
                    f,
                    "no XML files found in '{}'",
                    searched.display()
                )
            }
        }
    }
}

impl std::error::Error for LoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
