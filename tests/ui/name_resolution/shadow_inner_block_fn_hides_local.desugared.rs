fn main() -> () {
    let f1: bool;
    f1 = true;
    {
        fn main__f1() -> () {}
        main__f1();
    }
    crate::print(place_to_value!(f1));
}
