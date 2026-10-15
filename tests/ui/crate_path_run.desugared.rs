fn foo(x: &mut bool) -> () {
    *x = true;
}
fn main() -> () {
    let x: bool;
    x = false;
    crate::foo(&mut x);
    crate::print(place_to_value!(x));
}
