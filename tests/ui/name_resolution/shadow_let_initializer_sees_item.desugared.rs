fn f1() -> bool {
    true
}
fn main() -> () {
    let f1: bool;
    f1 = crate::f1();
    crate::print(place_to_value!(f1));
}
