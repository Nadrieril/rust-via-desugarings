// Path expressions which resolve to functions, not in callee position.
fn foo() {}

fn main() {
    let a = foo;
    let b = (foo, { foo });
    let c = crate::foo;
    let d = (crate::foo, { crate::main });
    fn bar() {}
    let e = bar;
    let f = (bar, { bar });
}
