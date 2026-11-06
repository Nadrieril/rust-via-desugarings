fn main() -> () {
    let x: bool;
    x = true;
    fn main__f1() -> () {
        let x: bool;
        x = false;
        crate::print(place_to_value!(x));
    }
    fn main__f2(x: bool) -> () {
        crate::print(place_to_value!(x));
    }
    main__f1();
    main__f2(false);
    crate::print(place_to_value!(x));
}
