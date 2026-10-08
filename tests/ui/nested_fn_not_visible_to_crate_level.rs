//@ known-failure
fn main() {
    fn foo() {}
}

fn bar() {
    foo();
}
