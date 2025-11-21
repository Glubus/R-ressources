use std::path::PathBuf;

#[derive(Debug)]
pub enum ParserError {
    Xml { path: PathBuf, message: String },
}

impl std::fmt::Display for ParserError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Xml { path, message } => {
                write!(f, "{}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ParserError {}
