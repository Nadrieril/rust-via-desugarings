// A nested function used as a value.
fn main() {
    fn foo() {}
    let x = foo;
    let y = (foo, { foo });
}
