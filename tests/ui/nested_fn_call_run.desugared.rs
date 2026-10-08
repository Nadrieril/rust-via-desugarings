fn main() -> () {
    let x: bool;
    x = false;
    crate::main__foo(&mut x);
    crate::print(place_to_value!(x));
}
fn main__foo(x: &mut bool) -> () {
    *x = true;
}
