/* src/ast.rs */

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
    Variable {
        /// The content parts within the variable delimiters.
        parts: Vec<Self>,
    },
}

/// Configuration limits to prevent resource exhaustion attacks.
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
