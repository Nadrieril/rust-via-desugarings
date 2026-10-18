//@ expect-rejection: Invalid @ Desugaring
// A nested function isn't visible from other crate-level functions.
fn main() {
    fn foo() {}
}

fn bar() {
    foo();
}
