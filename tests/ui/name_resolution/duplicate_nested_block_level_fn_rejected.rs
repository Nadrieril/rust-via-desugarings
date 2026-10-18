//@ expect-rejection: Invalid @ Desugaring
// Two items with the same name are rejected in a block inside a nested function.
fn main() {
    fn outer() {
        {
            fn foo() {}
            fn foo() {}
        }
    }
}
