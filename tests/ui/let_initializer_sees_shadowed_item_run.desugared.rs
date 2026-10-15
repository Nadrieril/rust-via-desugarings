fn f1() -> () {
    crate::print(true);
}
fn main() -> () {
    let f1: ();
    f1 = crate::f1();
}
