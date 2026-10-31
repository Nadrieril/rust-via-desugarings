fn main() -> () {
    let x: bool;
    x = false;
    crate::main__foo(&mut x);
    crate::print(place_to_value!(x));
}
fn main__foo(x: &mut bool) -> () {
    crate::main__bar(place_to_value!(x));
}
fn main__bar(x: &mut bool) -> () {
    *x = true;
}
