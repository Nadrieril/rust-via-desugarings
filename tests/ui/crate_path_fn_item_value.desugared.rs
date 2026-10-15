fn foo() -> () {}
fn main() -> () {
    let x;
    x = crate::foo;
    let y;
    y = (crate::foo, {
        crate::main
    });
}
