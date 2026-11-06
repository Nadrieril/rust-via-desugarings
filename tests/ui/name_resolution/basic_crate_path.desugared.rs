fn foo() -> () {}
fn main() -> () {
    fn main__foo() -> () {}
    fn main__bar() -> () {
        crate::foo();
    }
    crate::foo();
    let foo: bool;
    foo = false;
    crate::foo();
}
