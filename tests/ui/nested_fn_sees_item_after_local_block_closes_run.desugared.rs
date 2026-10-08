fn f1() -> () {
    crate::print(true);
}
fn main() -> () {
    if true {
        let f1: bool;
        f1 = false;
        crate::print(place_to_value!(f1));
    } else {}
    crate::main__f2();
}
fn main__f2() -> () {
    crate::f1();
}
