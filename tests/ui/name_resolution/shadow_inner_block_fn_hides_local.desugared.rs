fn main() -> () {
    let f1: bool;
    f1 = true;
    {
        fn f1() -> () {}
        f1();
    }
    crate::print(place_to_value!(f1));
}
