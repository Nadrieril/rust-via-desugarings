fn main() -> () {
    crate::main__f2(true);
}
fn main__f1() -> () {}
fn main__f2(f1: bool) -> () {
    crate::print(place_to_value!(f1));
}
