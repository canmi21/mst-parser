//! Nested variable example.

use mst_parser::parse;

fn main() {
    let input = "Config: {{service.{{env}}.port}}";
    println!("Input: {}", input);

    // This parses as:
    // Text("Config: ")
    // Variable(
    //   Text("service."),
    //   Variable(Text("env")),
    //   Text(".port")
    // )
    let nodes = parse(input).unwrap();
    println!("{:#?}", nodes);
}