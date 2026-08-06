//! The binary.
//!
//! It prints its version and exits. Everything else is behind an issue.

fn main() {
    println!("lichttisch {}", env!("CARGO_PKG_VERSION"));
}
