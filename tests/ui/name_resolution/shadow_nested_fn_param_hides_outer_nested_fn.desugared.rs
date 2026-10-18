fn main() -> () {
    fn f1() -> () {}
    fn f2(f1: bool) -> () {
        crate::print(place_to_value!(f1));
    }
}
