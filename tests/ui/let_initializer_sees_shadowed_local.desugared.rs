fn main() -> () {
    let x: bool;
    x = true;
    {
        let x: bool;
        x = place_to_value!(x);
        crate::print(place_to_value!(x));
    }
}
