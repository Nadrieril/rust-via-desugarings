fn main() -> () {
    crate::main__f1();
    let f1: bool;
    f1 = true;
    crate::print(place_to_value!(f1));
}
fn main__f1() -> () {}
fn other() -> () {
    crate::other__f2();
    let f2: bool;
    f2 = true;
    crate::print(place_to_value!(f2));
}
fn other__f2() -> () {}
