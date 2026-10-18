fn f1() -> () {}
fn main() -> () {
    if true {
        let f1: bool;
        f1 = false;
        crate::print(place_to_value!(f1));
    } else {}
    fn f2() -> () {
        crate::f1();
    }
}
