/* examples/limits.rs */

//! Limits configuration example.

use mst_parser::{Limits, Parser};

fn main() {
	// 1. Strict depth limit
	let input_deep = "{{a{{b}}}}";
	let strict_depth = Limits {
		max_depth: 1,
		..Limits::default()
	};
	let parser_depth = Parser::new(strict_depth);

	println!("Parsing '{input_deep}' with max_depth=1...");
	match parser_depth.parse(input_deep) {
		Ok(_) => println!("Unexpected success!"),
		Err(e) => println!("Caught expected error: {e}"),
	}

	// 2. Strict node limit
	let input_long = "a{{b}}c{{d}}e";
	// "a", {{b}}, "b", "c", {{d}}, "d", "e" -> many nodes
	let strict_nodes = Limits {
		max_nodes: 3,
		..Limits::default()
	};
	let parser_nodes = Parser::new(strict_nodes);

	println!("\nParsing '{input_long}' with max_nodes=3...");
	match parser_nodes.parse(input_long) {
		Ok(_) => println!("Unexpected success!"),
		Err(e) => println!("Caught expected error: {e}"),
	}
}
