fn main() -> () {
    let x: bool;
    x = {
        crate::main__f()
    };
    crate::print(place_to_value!(x));
}
fn main__f() -> bool {
    true
}
