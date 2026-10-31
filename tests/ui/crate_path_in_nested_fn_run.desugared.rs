fn foo(x: &mut bool) -> () {
    *x = true;
}
fn main() -> () {
    let x: bool;
    x = false;
    crate::main__bar(&mut x);
    crate::print(place_to_value!(x));
}
fn main__bar(x: &mut bool) -> () {
    crate::foo(place_to_value!(x));
}
