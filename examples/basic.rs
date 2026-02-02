/* examples/basic.rs */

//! Basic usage example.

use mst_parser::parse;

fn main() {
    let input = "Hello {{name}}!";
    println!("Input: {}", input);

    match parse(input) {
        Ok(nodes) => {
            println!("AST:");
            for node in nodes {
                println!("  {:?}", node);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}