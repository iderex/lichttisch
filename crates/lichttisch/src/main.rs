//! The binary.
//!
//! It prints its version and exits. Everything else is behind an issue.
//!
//! The line it prints is built by a function rather than composed inside
//! `main`, so that the suite has something to judge. A binary whose only
//! behaviour cannot be reached from a test is a module with no tests at all,
//! which is what the coverage floor in #17 exists to catch, and the first
//! module to be caught by it should not be this one.

/// The first thing an operator ever sees from this program.
fn banner() -> String {
    format!("lichttisch {}", env!("CARGO_PKG_VERSION"))
}

fn main() {
    println!("{}", banner());
}

#[cfg(test)]
mod tests {
    use super::banner;

    /// What the banner promises is that a person reading a terminal, or a bug
    /// report quoting one, can tell which program produced the line and which
    /// build of it. Asserted as those two properties rather than against a
    /// copy of the format string, which would pass whatever the format string
    /// said.
    #[test]
    fn the_banner_names_the_program_and_a_three_part_version() {
        let banner = banner();

        let version = banner.strip_prefix("lichttisch ").unwrap_or_else(|| {
            panic!("the banner does not name the program it came from:\n\n    {banner}\n")
        });

        let parts: Vec<&str> = version.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "the banner carries a version with {} parts rather than three, so it \
             does not say which build produced it:\n\n    {banner}\n",
            parts.len()
        );
        assert!(
            parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())),
            "the banner carries a version part that is not a number:\n\n    {banner}\n"
        );
    }
}
