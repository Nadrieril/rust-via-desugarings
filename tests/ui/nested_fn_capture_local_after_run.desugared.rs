fn f1() -> () {
    crate::print(true);
}
fn main() -> () {
    crate::main__f2();
    let f1: bool;
    f1 = false;
}
fn main__f2() -> () {
    crate::f1();
}
