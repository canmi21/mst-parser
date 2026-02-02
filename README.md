# MST Parser

A zero-dependency, mustache-style template parser supporting nested variables.

`mst-parser` provides a robust recursive descent parser for `{{variable}}` style syntax. It produces an Abstract Syntax Tree (AST) suitable for template engines or configuration processors, with built-in protections against nested recursion.

## Features

- **Nested Variables**: Supports complex nested structures like `{{ key.{{subsection}} }}`.
- **Safety**: Configurable limits on recursion depth and node count to prevent malicious inputs.
- **Zero Dependency**: Lightweight implementation with no mandatory external dependencies.
- **no_std Support**: Fully compatible with `#![no_std]` environments (requires `alloc`).
- **Diagnostics**: Optional `tracing` integration for detailed parser execution logs.

## Usage Examples

Check the `examples` directory for runnable code:

- **Basic Usage**: [`examples/basic.rs`](examples/basic.rs) - Parse a simple template string into an AST.
- **Nested Variables**: [`examples/nested.rs`](examples/nested.rs) - Demonstrate deep variable nesting.
- **Safety Limits**: [`examples/limits.rs`](examples/limits.rs) - Enforce parser depth and node limits.
- **Tracing**: [`examples/tracing.rs`](examples/tracing.rs) - Enable and configure diagnostic logging.

## Installation

```toml
[dependencies]
mst-parser = { version = "0.1", features = ["full"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `tracing` | Enables logging and diagnostic instrumentation via the `tracing` crate. |
| `std` | Enables standard library support for error handling and formatting. |
| `full` | Enables all features above. |

## License

Released under the MIT License © 2026 [Canmi](https://github.com/canmi21)
