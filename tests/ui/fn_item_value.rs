fn foo() {}

fn f() {
    let x = foo;
    let y = (foo, foo);
    let z = { foo };
}
