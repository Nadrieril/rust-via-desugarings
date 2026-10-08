fn foo() -> () {}
fn f() -> () {
    let x;
    x = place_to_value!(crate::foo);
    let y;
    y = (place_to_value!(crate::foo), place_to_value!(crate::foo));
    let z;
    z = {
        place_to_value!(crate::foo)
    };
}
