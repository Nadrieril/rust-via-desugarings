fn foo() -> () {}
fn main() -> () {
    let a;
    a = crate::foo;
    let b;
    b = (crate::foo, {
        crate::foo
    });
    let c;
    c = crate::foo;
    let d;
    d = (crate::foo, {
        crate::main
    });
    let e;
    e = crate::main__bar;
    let f;
    f = (crate::main__bar, {
        crate::main__bar
    });
}
fn main__bar() -> () {}
