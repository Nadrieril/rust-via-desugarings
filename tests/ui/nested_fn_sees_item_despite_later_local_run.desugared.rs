fn main() -> () {
    let f1: bool;
    f1 = false;
    crate::main__f2();
}
fn main__f1() -> () {
    crate::print(true);
}
fn main__f2() -> () {
    crate::main__f1();
}
