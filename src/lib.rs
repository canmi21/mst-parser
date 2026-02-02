#![cfg_attr(not(feature = "std"), no_std)]
/* src/lib.rs */
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
//! use mst_parser::{parse, Limits, Node, Parser};
//!
//! let input = "Hello {{user.{{attr}}}}!";
//! // Use default limits
//! let nodes = parse(input).unwrap();
//!
//! assert_eq!(nodes.len(), 3);
//! match &nodes[1] {
//!     Node::Variable { parts } => {
//!         // parts represents: "user.", {{attr}}
//!         assert_eq!(parts.len(), 2);
//!     }
//!     _ => panic!("Expected variable"),
//! }
//!
//! // Or with custom limits
//! let limits = Limits { max_depth: 2, ..Limits::default() };
//! let parser = Parser::new(limits);
//! let nodes = parser.parse(input).unwrap();
//! ```

extern crate alloc;

mod ast;
mod error;
mod parser;

pub use ast::{Limits, Node};
pub use error::Error;
pub use parser::{Parser, parse};
