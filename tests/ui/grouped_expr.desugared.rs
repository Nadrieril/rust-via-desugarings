fn main() -> () {
    let x: bool;
    x = true;
    crate::print(place_to_value!(x));
    ();
}
