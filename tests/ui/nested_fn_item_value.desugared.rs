fn main() -> () {
    let x;
    x = place_to_value!(crate::main__foo);
    let y;
    y = (place_to_value!(crate::main__foo), {
        place_to_value!(crate::main__foo)
    });
}
fn main__foo() -> () {}
