/* src/error.rs */

/// Errors that can occur during template parsing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
	/// The recursion depth exceeded `Limits::max_depth`.
	#[error("recursion depth exceeded (limit: {limit}) at byte {offset}")]
	DepthExceeded {
		/// The depth limit that was exceeded.
		limit: usize,
		/// The byte offset where the error occurred.
		offset: usize,
	},

	/// The total number of AST nodes exceeded `Limits::max_nodes`.
	#[error("node limit exceeded (limit: {limit}) at byte {offset}")]
	NodeLimitExceeded {
		/// The node count limit that was exceeded.
		limit: usize,
		/// The byte offset where the error occurred.
		offset: usize,
	},

	/// A variable tag `{{` was opened but never closed with `}}`.
	#[error("unclosed variable tag at byte {offset}")]
	UnclosedVariable {
		/// The byte offset of the unclosed `{{`.
		offset: usize,
	},

	/// A variable tag was empty, e.g. `{{}}`.
	#[error("empty variable tag at byte {offset}")]
	EmptyVariable {
		/// The byte offset of the empty variable tag `{{`.
		offset: usize,
	},

	/// Unbalanced tag nesting detected.
	#[error("unbalanced tag nesting at byte {offset}")]
	UnbalancedTag {
		/// The byte offset where the imbalance was detected.
		offset: usize,
	},
}
