fn f2(f1: bool) -> () {
    crate::f2__f1();
}
fn f2__f1() -> () {
    crate::print(true);
}
fn main() -> () {
    crate::f2(false);
}
