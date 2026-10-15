fn f1() -> () {
    crate::print(true);
}
fn main() -> () {
    crate::f1();
    let f1: bool;
    f1 = false;
    crate::print(place_to_value!(f1));
}
