/* examples/basic.rs */

use mst_parser::{Limits, parse};

fn main() {
	let input = "Hello {{name}}!";
	println!("Input: {}", input);

	match parse(input, &Limits::default()) {
		Ok(nodes) => {
			println!("AST:");
			for node in nodes {
				println!("  {:?}", node);
			}
		}
		Err(e) => eprintln!("Error: {}", e),
	}
}
