fn f1() -> () {}
fn main() -> () {
    fn f2() -> () {
        crate::f1();
    }
    fn f3() -> () {}
    fn f4() -> () {
        f3();
    }
    let f1: bool;
    f1 = false;
    let f3: bool;
    f3 = false;
}
