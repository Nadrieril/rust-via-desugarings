fn main() -> () {
    let x: bool;
    x = false;
    let r: &bool;
    r = &x;
    let s: &mut bool;
    s = &mut x;
    crate::print(place_to_value!(*r));
}
