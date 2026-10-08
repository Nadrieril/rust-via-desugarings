//@ known-failure
fn foo() {}

fn main() {
    crate::foo::bar();
}
