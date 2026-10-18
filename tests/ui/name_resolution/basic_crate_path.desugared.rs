fn foo() -> () {}
fn main() -> () {
    fn foo() -> () {}
    fn bar() -> () {
        crate::foo();
    }
    crate::foo();
    let foo: bool;
    foo = false;
    crate::foo();
}
