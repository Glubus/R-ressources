//! XML parsing into Abstract Syntax Tree (AST).
//!
//! This module parses XML resource files into structured `ParsedResourceFile` objects.
//! Each file is parsed independently, producing an AST that will be merged into the
//! unified `ResourceGraph` in the IR stage.

mod ast;
mod error;
mod reader;

pub use ast::{
    ParsedResource, ParsedResourceFile, ResourceKind, ScalarValue,
};
pub use error::ParserError;

use crate::generator::input::RawResourceFile;

/// Parse a list of preprocessed raw files into structured resources.
pub fn parse_raw_files(
    raw_files: &[RawResourceFile],
) -> Result<Vec<ParsedResourceFile>, ParserError> {
    raw_files.iter().map(reader::parse_single_file).collect()
}
