fn foo() -> () {}
fn f() -> () {
    let x;
    x = crate::foo;
    let y;
    y = (crate::foo, crate::foo);
    let z;
    z = {
        crate::foo
    };
}
