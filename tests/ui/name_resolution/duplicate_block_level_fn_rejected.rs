//@ expect-rejection: Invalid @ Desugaring
// Two items with the same name in one block are rejected.
fn main() {
    fn foo() {}
    fn foo() {}
}
