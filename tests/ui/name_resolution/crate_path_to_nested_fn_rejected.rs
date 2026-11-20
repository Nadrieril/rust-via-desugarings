//@ expect-rejection: Invalid @ Desugaring
// A crate-relative path expression doesn't resolve to a nested function.
fn main() {
    fn foo() {}
    crate::foo();
}
