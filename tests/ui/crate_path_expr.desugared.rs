fn foo() -> bool {
    true
}
fn main() -> () {
    crate::foo();
    let x: bool;
    x = crate::foo();
    let y;
    y = (crate::foo, {
        crate::foo
    });
}
fn main__bar() -> () {
    crate::foo();
}
