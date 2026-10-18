fn main() -> () {
    fn f1() -> () {}
    f1();
    let f1: bool;
    f1 = true;
    crate::print(place_to_value!(f1));
}
fn other() -> () {
    f2();
    let f2: bool;
    f2 = true;
    fn f2() -> () {}
    crate::print(place_to_value!(f2));
}
