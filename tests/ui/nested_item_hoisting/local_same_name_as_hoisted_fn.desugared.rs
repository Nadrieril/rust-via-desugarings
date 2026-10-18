fn main() -> () {
    crate::print(crate::main__f());
    let f: bool;
    f = false;
    crate::print(place_to_value!(f));
}
fn main__f() -> bool {
    true
}
