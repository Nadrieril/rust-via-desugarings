fn main() -> () {
    let x: bool;
    x = true;
    crate::main__f1();
    crate::print(place_to_value!(x));
}
fn main__f1() -> () {
    let x: bool;
    x = false;
    crate::print(place_to_value!(x));
}
