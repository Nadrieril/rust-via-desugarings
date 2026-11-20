//@ expect-rejection: Invalid @ Desugaring
// A crate-relative path expression doesn't resolve to a local.
fn main() {
    let x: bool = true;
    crate::x;
}
