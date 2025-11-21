//! Flat module generation (`r::` style).
//!
//! This module generates the flat `r::` namespace structure from the resource graph.
//! Resources are organized in nested modules matching their XML namespace hierarchy.
//!
//! Example:
//! ```xml
//! <ns name="auth">
//!     <string name="title">Login</string>
//! </ns>
//! ```
//!
//! Generates:
//! ```rust
//! pub mod r {
//!     pub mod auth {
//!         pub const TITLE: &str = "Login";
//!     }
//! }
//! ```

mod emitter;
mod tree;

pub use emitter::generate_r_module;

