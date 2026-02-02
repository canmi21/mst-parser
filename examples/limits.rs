/* examples/limits.rs */

use mst_parser::{Limits, parse};

fn main() {
	// 1. Strict depth limit
	let input_deep = "{{a{{b}}}}";
	let strict_depth = Limits {
		max_depth: 1,
		..Limits::default()
	};

	println!("Parsing '{}' with max_depth=1...", input_deep);
	match parse(input_deep, &strict_depth) {
		Ok(_) => println!("Unexpected success!"),
		Err(e) => println!("Caught expected error: {}", e),
	}

	// 2. Strict node limit
	let input_long = "a{{b}}c{{d}}e";
	// "a", {{b}}, "b", "c", {{d}}, "d", "e" -> many nodes
	let strict_nodes = Limits {
		max_nodes: 3,
		..Limits::default()
	};

	println!("\nParsing '{}' with max_nodes=3...", input_long);
	match parse(input_long, &strict_nodes) {
		Ok(_) => println!("Unexpected success!"),
		Err(e) => println!("Caught expected error: {}", e),
	}
}
