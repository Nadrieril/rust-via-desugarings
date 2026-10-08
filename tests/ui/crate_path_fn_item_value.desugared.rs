fn foo() -> () {}
fn main() -> () {
    let x;
    x = place_to_value!(crate::foo);
    let y;
    y = (place_to_value!(crate::foo), {
        place_to_value!(crate::main)
    });
}
