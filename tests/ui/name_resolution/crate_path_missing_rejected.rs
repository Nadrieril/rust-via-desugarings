//@ expect-rejection: Invalid @ Desugaring
// A crate-relative path expression which doesn't name a crate-level item is rejected.
fn main() {
    crate::foo();
}
