fn foo() -> () {}
fn main() -> () {
    crate::foo();
    let foo: bool;
    foo = false;
    crate::foo();
}
fn main__foo() -> () {}
fn main__bar() -> () {
    crate::foo();
}
