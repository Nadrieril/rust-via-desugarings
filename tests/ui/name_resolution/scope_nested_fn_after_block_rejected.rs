//@ expect-rejection: Invalid @ Desugaring
// A nested function isn't visible after the end of its block.
fn main() {
    {
        fn foo() {}
    }
    foo();
}
