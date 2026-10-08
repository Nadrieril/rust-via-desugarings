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
    let e;
    e = place_to_value!(crate::main__bar);
    let f;
    f = (place_to_value!(crate::main__bar), {
        place_to_value!(crate::main__bar)
    });
}
fn main__bar() -> () {}
