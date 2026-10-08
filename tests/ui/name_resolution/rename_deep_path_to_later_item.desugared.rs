fn main() -> () {
    crate::main__outer();
}
fn main__outer() -> () {
    crate::main__outer__inner();
}
fn main__outer__inner() -> () {
    crate::main__later();
}
fn main__later() -> () {}
