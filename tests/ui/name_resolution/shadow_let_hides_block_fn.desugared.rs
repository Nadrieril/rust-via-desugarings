fn main() -> () {
    fn main__f1() -> () {}
    main__f1();
    let f1: bool;
    f1 = true;
    crate::print(place_to_value!(f1));
}
fn other() -> () {
    other__f2();
    let f2: bool;
    f2 = true;
    fn other__f2() -> () {}
    crate::print(place_to_value!(f2));
}
