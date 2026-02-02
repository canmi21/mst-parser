/* src/parser.rs */

use crate::ast::{Limits, Node};
use crate::error::Error;
use alloc::string::String;
use alloc::vec::Vec;

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

	fn inc_node(&mut self, offset: usize) -> Result<(), Error> {
		if self.node_count >= self.max_nodes {
			return Err(Error::NodeLimitExceeded {
				limit: self.max_nodes,
				offset,
			});
		}
		self.node_count += 1;
		Ok(())
	}

	fn enter_depth(&mut self, offset: usize) -> Result<(), Error> {
		if self.depth >= self.max_depth {
			return Err(Error::DepthExceeded {
				limit: self.max_depth,
				offset,
			});
		}
		self.depth += 1;
		Ok(())
	}

	fn exit_depth(&mut self, offset: usize) -> Result<(), Error> {
		self.depth = self
			.depth
			.checked_sub(1)
			.ok_or(Error::UnbalancedTag { offset })?;
		Ok(())
	}
}

/// A configured template parser.
#[derive(Debug, Clone, Default)]
pub struct Parser {
	limits: Limits,
}

impl Parser {
	/// Creates a new parser with the given limits.
	#[must_use]
	pub fn new(limits: Limits) -> Self {
		Self { limits }
	}

	/// Parses a template string into AST nodes.
	pub fn parse(&self, input: &str) -> Result<Vec<Node>, Error> {
		parse_inner(input, &self.limits)
	}
}

/// Convenience function that parses with default limits.
pub fn parse(input: &str) -> Result<Vec<Node>, Error> {
	Parser::default().parse(input)
}

fn parse_inner(input: &str, limits: &Limits) -> Result<Vec<Node>, Error> {
	let mut state = ParserState::new(limits);
	let mut chars = input.char_indices().peekable();
	parse_recursive(&mut chars, input, &mut state, None)
}

/// Internal recursive parser.
///
/// `var_open_offset`: Some(idx) if we are currently parsing content inside `{{ ... }}` started at `idx`.
fn parse_recursive(
	chars: &mut core::iter::Peekable<core::str::CharIndices<'_>>,
	original_input: &str,
	state: &mut ParserState,
	var_open_offset: Option<usize>,
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
						state.inc_node(start)?;
						nodes.push(Node::Text(String::from(text)));
						#[cfg(feature = "tracing")]
						tracing::trace!(text = text, "parsed text node");
					}
					current_text_start = None;
				}

				// 2. Consume the second '{'
				chars.next();

				// 3. Parse variable content recursively
				state.inc_node(idx)?; // Count the Variable node itself
				state.enter_depth(idx)?;

				#[cfg(feature = "tracing")]
				tracing::debug!(depth = state.depth, "entering variable");

				let parts = parse_recursive(chars, original_input, state, Some(idx))?;

				#[cfg(feature = "tracing")]
				tracing::debug!(depth = state.depth, "exiting variable");

				state.exit_depth(idx)?;
				nodes.push(Node::Variable { parts });

				continue;
			}
		}

		// Check for '}}' end sequence
		if let Some(open_offset) = var_open_offset {
			if ch == '}' {
				if let Some(&(_, '}')) = chars.peek() {
					// Found "}}", end of current variable

					// 1. Flush any preceding text
					if let Some(start) = current_text_start {
						if start < idx {
							let text = &original_input[start..idx];
							state.inc_node(start)?;
							nodes.push(Node::Text(String::from(text)));
							#[cfg(feature = "tracing")]
							tracing::trace!(text = text, "parsed text node");
						}
						// current_text_start = None; // Not needed before return
					}

					// 2. Consume the second '}'
					chars.next();

					// Check for empty variable
					if nodes.is_empty() {
						return Err(Error::EmptyVariable {
							offset: open_offset,
						});
					}

					// Return what we collected for this variable
					return Ok(nodes);
				}
			}
		}

		// Normal character, track text start
		if current_text_start.is_none() {
			current_text_start = Some(idx);
		}
	}

	// End of input reached
	if let Some(open_offset) = var_open_offset {
		// If we were expecting '}}' but hit EOF, that's an error
		return Err(Error::UnclosedVariable {
			offset: open_offset,
		});
	}

	// Flush remaining text at EOF for top-level
	if let Some(start) = current_text_start {
		let end_idx = original_input.len();
		if start < end_idx {
			let text = &original_input[start..end_idx];
			state.inc_node(start)?;
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
	use alloc::vec;

	#[test]
	fn test_plain_text() {
		let input = "Hello World";
		let nodes = parse(input).unwrap();
		assert_eq!(nodes, vec![Node::Text("Hello World".into())]);
	}

	#[test]
	fn test_simple_variable() {
		let input = "Hello {{name}}!";
		let nodes = parse(input).unwrap();
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
		let nodes = parse(input).unwrap();

		match &nodes[0] {
			Node::Variable { parts } => {
				assert_eq!(parts.len(), 2);
				assert_eq!(parts[0], Node::Text("key.".into()));
				match &parts[1] {
					Node::Variable { parts: sub_parts } => {
						assert_eq!(sub_parts[0], Node::Text("sub".into()));
					}
					Node::Text(_) => panic!("Expected inner variable"),
				}
			}
			Node::Text(_) => panic!("Expected outer variable"),
		}
	}

	#[test]
	fn test_unclosed_variable() {
		let input = "Hello {{name";
		let err = parse(input).unwrap_err();
		match err {
			Error::UnclosedVariable { offset } => assert_eq!(offset, 6),
			_ => panic!("Unexpected error: {err:?}"),
		}
	}

	#[test]
	fn test_depth_limit() {
		let limits = Limits {
			max_depth: 1,
			max_nodes: 100,
		};
		let parser = Parser::new(limits);
		// Depth 0 (root) -> Depth 1 ({{a}}) -> Depth 2 ({{b}}) -> Fail
		let input = "{{a{{b}}}}";
		let err = parser.parse(input).unwrap_err();
		match err {
			Error::DepthExceeded { limit, offset } => {
				assert_eq!(limit, 1);
				assert_eq!(offset, 3); // second {{ starts at 3
			}
			_ => panic!("Unexpected error: {err:?}"),
		}
	}

	#[test]
	fn test_node_limit() {
		let limits = Limits {
			max_depth: 10,
			max_nodes: 2,
		};
		let parser = Parser::new(limits);
		// "abc" (1 node) + {{...}} (1 node) + "d" (1 node) = 3 nodes -> Fail
		let input = "abc{{d}}";
		let err = parser.parse(input).unwrap_err();
		match err {
			Error::NodeLimitExceeded { limit, offset } => {
				assert_eq!(limit, 2); // Error happens at "d" (node 3)
				assert_eq!(offset, 5); // "d" starts at 5
			}
			_ => panic!("Unexpected error: {err:?}"),
		}
	}

	#[test]
	fn test_consecutive_braces() {
		// "{{{" -> First "{{" opens variable. Remaining "{" is text inside variable.
		// Expecting "}}" to close.
		let input = "{{{}}";
		let nodes = parse(input).unwrap();
		match &nodes[0] {
			Node::Variable { parts } => {
				assert_eq!(parts[0], Node::Text("{".into()));
			}
			Node::Text(_) => panic!("Expected variable"),
		}
	}

	#[test]
	fn test_empty_variable() {
		let input = "{{}}";
		let err = parse(input).unwrap_err();
		match err {
			Error::EmptyVariable { offset } => assert_eq!(offset, 0),
			_ => panic!("Unexpected error: {err:?}"),
		}
	}

	#[test]
	fn test_top_level_closing_braces() {
		let input = "hello}}world";
		let nodes = parse(input).unwrap();
		assert_eq!(nodes, vec![Node::Text("hello}}world".into())]);
	}
}
