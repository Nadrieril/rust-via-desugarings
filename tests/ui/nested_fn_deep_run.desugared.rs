fn main() -> () {
    let x: bool;
    x = false;
    crate::main__f2(&mut x);
    crate::print(place_to_value!(x));
}
fn main__f1(x: &mut bool) -> () {
    *x = true;
}
fn main__f2(x: &mut bool) -> () {
    crate::main__f2__f3(place_to_value!(x));
}
fn main__f2__f3(x: &mut bool) -> () {
    crate::main__f1(place_to_value!(x));
}
