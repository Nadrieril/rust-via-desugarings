fn foo() -> () {}
fn bar() -> () {
    crate::foo();
}
fn main() -> () {
    {
        crate::main__foo_0();
    }
    crate::main__foo();
}
fn main__foo() -> () {}
fn main__foo_0() -> () {}
fn main__baz() -> () {
    crate::main__foo_0();
}
