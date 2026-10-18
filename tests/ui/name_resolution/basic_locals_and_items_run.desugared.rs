fn f1() -> () {
    crate::print(true);
}
fn f2(f1: bool) -> () {
    crate::print(place_to_value!(f1));
}
fn main() -> () {
    crate::f1();
    let f1: bool;
    f1 = false;
    crate::print(place_to_value!(f1));
    crate::f1();
    crate::f2(false);
}
