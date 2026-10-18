fn foo() -> () {}
fn main() -> () {
    let a;
    a = place_to_value!(crate::foo);
    let b;
    b = (place_to_value!(crate::foo), {
        place_to_value!(crate::foo)
    });
    let c;
    c = place_to_value!(crate::foo);
    let d;
    d = (place_to_value!(crate::foo), {
        place_to_value!(crate::main)
    });
    fn bar() -> () {}
    let e;
    e = place_to_value!(bar);
    let f;
    f = (place_to_value!(bar), {
        place_to_value!(bar)
    });
}
