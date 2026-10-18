fn main() -> () {
    let x: bool;
    x = true;
    fn f1() -> () {
        let x: bool;
        x = false;
        crate::print(place_to_value!(x));
    }
    fn f2(x: bool) -> () {
        crate::print(place_to_value!(x));
    }
    f1();
    f2(false);
    crate::print(place_to_value!(x));
}
