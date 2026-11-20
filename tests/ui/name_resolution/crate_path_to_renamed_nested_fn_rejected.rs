//@ expect-rejection: Invalid @ Desugaring
// A crate-relative path expression can't resolve by "guessing" a nested function's unambiguous
// name.
fn main() {
    fn foo() {}
    crate::main__foo();
}
