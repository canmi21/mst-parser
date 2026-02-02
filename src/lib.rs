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

use alloc::string::String;
use alloc::vec::Vec;

/// Represents a node in the parsed template AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
	/// A literal text segment.
	Text(String),
	/// A variable segment, e.g., `{{ ... }}`.
	///
	/// The `parts` vector contains the segments inside the variable tag.
	/// For example, `{{ a.{{b}} }}` parses to a `Variable` containing:
	/// - `Text("a.")`
	/// - `Variable { parts: [Text("b")] }`
	Variable {
		/// The content parts within the variable delimiters.
		parts: Vec<Self>,
	},
}

/// Configuration limits to prevent resource exhaustion attacks (e.g., stack overflow, OOM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
	/// Maximum allowed nesting depth of variables. Default is 5.
	pub max_depth: usize,
	/// Maximum allowed total nodes in the AST. Default is 50.
	pub max_nodes: usize,
}

impl Default for Limits {
	fn default() -> Self {
		Self {
			max_depth: 5,
			max_nodes: 50,
		}
	}
}

/// Errors that can occur during parsing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// The recursion depth exceeded `Limits::max_depth`.
	#[error("recursion depth exceeded (limit: {0})")]
	DepthExceeded(usize),

	/// The total number of AST nodes exceeded `Limits::max_nodes`.
	#[error("node limit exceeded (limit: {0})")]
	NodeLimitExceeded(usize),

	/// A variable tag `{{` was opened but never closed with `}}`.
	#[error("unclosed variable tag")]
	UnclosedVariable,
}

/// State object for the parser to track current limits.
struct ParserState {
	node_count: usize,
	depth: usize,
	max_nodes: usize,
	max_depth: usize,
}

impl ParserState {
	fn new(limits: &Limits) -> Self {
		Self {
			node_count: 0,
			depth: 0,
			max_nodes: limits.max_nodes,
			max_depth: limits.max_depth,
		}
	}

	fn inc_node(&mut self) -> Result<(), Error> {
		if self.node_count >= self.max_nodes {
			return Err(Error::NodeLimitExceeded(self.max_nodes));
		}
		self.node_count += 1;
		Ok(())
	}

	fn enter_depth(&mut self) -> Result<(), Error> {
		if self.depth >= self.max_depth {
			return Err(Error::DepthExceeded(self.max_depth));
		}
		self.depth += 1;
		Ok(())
	}

	fn exit_depth(&mut self) {
		if self.depth > 0 {
			self.depth -= 1;
		}
	}
}

/// Parses a template string into a list of AST nodes.
///
/// # Arguments
///
/// * `input` - The string to parse.
/// * `limits` - Security limits for the parser.
///
/// # Returns
///
/// * `Ok(Vec<Node>)` - A vector of top-level AST nodes.
/// * `Err(Error)` - If a limit is exceeded or syntax is invalid (e.g. unclosed tags).
#[cfg_attr(
	feature = "tracing",
	tracing::instrument(skip(input, limits), level = "debug")
)]
pub fn parse(input: &str, limits: &Limits) -> Result<Vec<Node>, Error> {
	let mut state = ParserState::new(limits);
	let mut chars = input.char_indices().peekable();
	parse_recursive(&mut chars, input, &mut state, false)
}

/// Internal recursive parser.
///
/// `is_inside_variable`: true if we are currently parsing content inside `{{ ... }}`.
fn parse_recursive(
	chars: &mut core::iter::Peekable<core::str::CharIndices<'_>>,
	original_input: &str,
	state: &mut ParserState,
	is_inside_variable: bool,
) -> Result<Vec<Node>, Error> {
	let mut nodes = Vec::new();
	let mut current_text_start = None;

	while let Some((idx, ch)) = chars.next() {
		// Check for '{{' start sequence
		if ch == '{' {
			if let Some(&(_, '{')) = chars.peek() {
				// Found "{{", start of a variable

				// 1. Flush any preceding text
				if let Some(start) = current_text_start {
					if start < idx {
						let text = &original_input[start..idx];
						state.inc_node()?;
						nodes.push(Node::Text(String::from(text)));
						#[cfg(feature = "tracing")]
						tracing::trace!(text = text, "parsed text node");
					}
					current_text_start = None;
				}

				// 2. Consume the second '{'
				chars.next();

				// 3. Parse variable content recursively
				state.inc_node()?; // Count the Variable node itself
				state.enter_depth()?;

				#[cfg(feature = "tracing")]
				tracing::debug!(depth = state.depth, "entering variable");

				let parts = parse_recursive(chars, original_input, state, true)?;

				#[cfg(feature = "tracing")]
				tracing::debug!(depth = state.depth, "exiting variable");

				state.exit_depth();
				nodes.push(Node::Variable { parts });

				continue;
			}
		}

		// Check for '}}' end sequence
		if is_inside_variable && ch == '}' {
			if let Some(&(_, '}')) = chars.peek() {
				// Found "}}", end of current variable

				// 1. Flush any preceding text
				if let Some(start) = current_text_start {
					if start < idx {
						let text = &original_input[start..idx];
						state.inc_node()?;
						nodes.push(Node::Text(String::from(text)));
						#[cfg(feature = "tracing")]
						tracing::trace!(text = text, "parsed text node");
					}
					// current_text_start = None; // Not needed before return
				}

				// 2. Consume the second '}'
				chars.next();

				// Return what we collected for this variable
				return Ok(nodes);
			}
		}

		// Normal character, track text start
		if current_text_start.is_none() {
			current_text_start = Some(idx);
		}
	}

	// End of input reached
	if is_inside_variable {
		// If we were expecting '}}' but hit EOF, that's an error
		return Err(Error::UnclosedVariable);
	}

	// Flush remaining text at EOF for top-level
	if let Some(start) = current_text_start {
		let end_idx = original_input.len();
		if start < end_idx {
			let text = &original_input[start..end_idx];
			state.inc_node()?;
			nodes.push(Node::Text(String::from(text)));
			#[cfg(feature = "tracing")]
			tracing::trace!(text = text, "parsed text node");
		}
	}

	Ok(nodes)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_plain_text() {
		let input = "Hello World";
		let nodes = parse(input, &Limits::default()).unwrap();
		assert_eq!(nodes, vec![Node::Text("Hello World".into())]);
	}

	#[test]
	fn test_simple_variable() {
		let input = "Hello {{name}}!";
		let nodes = parse(input, &Limits::default()).unwrap();
		assert_eq!(
			nodes,
			vec![
				Node::Text("Hello ".into()),
				Node::Variable {
					parts: vec![Node::Text("name".into())]
				},
				Node::Text("!".into())
			]
		);
	}

	#[test]
	fn test_nested_variable() {
		// "key.{{sub}}" -> Variable(Text("key."), Variable(Text("sub")))
		let input = "{{key.{{sub}}}}";
		let nodes = parse(input, &Limits::default()).unwrap();

		match &nodes[0] {
			Node::Variable { parts } => {
				assert_eq!(parts.len(), 2);
				assert_eq!(parts[0], Node::Text("key.".into()));
				match &parts[1] {
					Node::Variable { parts: sub_parts } => {
						assert_eq!(sub_parts[0], Node::Text("sub".into()));
					}
					_ => panic!("Expected inner variable"),
				}
			}
			_ => panic!("Expected outer variable"),
		}
	}

	#[test]
	fn test_unclosed_variable() {
		let input = "Hello {{name";
		let err = parse(input, &Limits::default()).unwrap_err();
		assert!(matches!(err, Error::UnclosedVariable));
	}

	#[test]
	fn test_depth_limit() {
		let limits = Limits {
			max_depth: 1,
			max_nodes: 100,
		};
		// Depth 0 (root) -> Depth 1 ({{a}}) -> Depth 2 ({{b}}) -> Fail
		let input = "{{a{{b}}}}";
		let err = parse(input, &limits).unwrap_err();
		assert!(matches!(err, Error::DepthExceeded(1)));
	}

	#[test]
	fn test_node_limit() {
		let limits = Limits {
			max_depth: 10,
			max_nodes: 2,
		};
		// "abc" (1 node) + {{...}} (1 node) + "d" (1 node) = 3 nodes -> Fail
		let input = "abc{{d}}";
		// Note: implementation details determine exact count.
		// "abc" = 1
		// Variable = 2 (limit reached?)
		// Inside variable "d" = 3
		let err = parse(input, &limits).unwrap_err();
		assert!(matches!(err, Error::NodeLimitExceeded(_)));
	}

	#[test]
	fn test_consecutive_braces() {
		// Should parse as text "{{{" -> starts variable, next char is '{' which is text inside?
		// Wait, standard mustache: "{{{" usually means unescaped.
		// But spec here is just recursive {{...}}.
		// "{{{" -> First "{{" opens variable. Remaining "{" is text inside variable.
		// Expecting "}}" to close.
		let input = "{{{}}";
		// "{{" opens. Inside: "{", then "}}" closes.
		let nodes = parse(input, &Limits::default()).unwrap();
		match &nodes[0] {
			Node::Variable { parts } => {
				assert_eq!(parts[0], Node::Text("{".into()));
			}
			_ => panic!("Expected variable"),
		}
	}
}
