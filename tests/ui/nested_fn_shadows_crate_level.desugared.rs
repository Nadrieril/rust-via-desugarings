fn foo() -> () {}
fn bar() -> () {
    crate::foo();
}
fn main() -> () {
    crate::main__foo();
}
fn main__foo() -> () {}
fn main__f1() -> () {
    crate::main__foo();
}
