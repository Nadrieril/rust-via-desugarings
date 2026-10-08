fn foo() -> bool {
    true
}
fn main() -> () {
    crate::foo();
    let x: bool;
    x = crate::foo();
    let y;
    y = (place_to_value!(crate::foo), {
        place_to_value!(crate::foo)
    });
}
fn main__bar() -> () {
    crate::foo();
}
