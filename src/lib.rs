/* src/lib.rs */

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![allow(clippy::collapsible_if)]

//! # mst-parser
//!
//! A zero-dependency, mustache-style template parser supporting nested variables.
//!
//! This crate provides a recursive descent parser for `{{variable}}` style syntax.
//! It produces an Abstract Syntax Tree (AST) but does *not* perform any rendering or substitution.
//!
//! ## Features
//!
//! * **Nested Variables**: Supports `{{ key.{{subsection}} }}` syntax.
//! * **Safety**: Configurable limits on recursion depth and node count to prevent malicious inputs.
//! * **no_std**: Compatible with `#![no_std]` environments (requires `alloc`).
//! * **Diagnostics**: Optional `tracing` integration for parser debugging.
//!
//! ## Example
//!
//! ```rust
//! use mst_parser::{parse, Limits, Node};
//!
//! let input = "Hello {{user.{{attr}}}}!";
//! let nodes = parse(input, &Limits::default()).unwrap();
//!
//! assert_eq!(nodes.len(), 3);
//! match &nodes[1] {
//!     Node::Variable { parts } => {
//!         // parts represents: "user.", {{attr}}
//!         assert_eq!(parts.len(), 2);
//!     }
//!     _ => panic!("Expected variable"),
//! }
//! ```

extern crate alloc;

mod ast;
mod error;
mod parser;

pub use ast::{Limits, Node};
pub use error::Error;
pub use parser::parse;
