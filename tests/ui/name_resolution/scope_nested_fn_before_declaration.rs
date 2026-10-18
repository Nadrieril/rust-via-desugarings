// A nested function is visible throughout its block, including before its declaration.
fn main() {
    foo();
    fn foo() {}
}
