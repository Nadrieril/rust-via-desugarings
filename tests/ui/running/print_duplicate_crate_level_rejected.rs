//@ expect-rejection: Invalid @ Desugaring
// A crate-level function named `print` is rejected, because the runners provide one.
fn print(x: bool) {}

fn main() {}
