/* examples/tracing.rs */

//! Run with: RUST_LOG=trace cargo run --example tracing --features full

use mst_parser::{Limits, parse};

fn main() {
	#[cfg(feature = "tracing")]
	{
		use tracing_subscriber::fmt::format::FmtSpan;
		tracing_subscriber::fmt()
			.with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
			.init();

		println!("Tracing initialized. Parsing...");
		let input = "Hello {{user.{{id}}}}";
		let _ = parse(input, &Limits::default());
	}

	#[cfg(not(feature = "tracing"))]
	{
		println!("Please run this example with 'full' or 'tracing' feature enabled:");
		println!("cargo run --example tracing --features full");
	}
}
