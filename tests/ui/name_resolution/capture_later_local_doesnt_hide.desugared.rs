fn f1() -> () {}
fn main() -> () {
    let f1: bool;
    f1 = false;
    let f3: bool;
    f3 = false;
}
fn main__f2() -> () {
    crate::f1();
}
fn main__f3() -> () {}
fn main__f4() -> () {
    crate::main__f3();
}
