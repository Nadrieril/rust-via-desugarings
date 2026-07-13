fn foo(x: &mut bool) -> () {
    *x = true;
}
fn main() -> () {
    let x: bool;
    x = false;
    foo(&mut x);
    print(place_to_value!(x));
}
