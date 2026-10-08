fn f1() -> () {
    crate::print(true);
}
fn main() -> () {
    let f1: bool;
    f1 = false;
    crate::f1();
    crate::print(place_to_value!(f1));
}
