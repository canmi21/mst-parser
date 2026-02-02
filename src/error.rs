/* src/error.rs */

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
