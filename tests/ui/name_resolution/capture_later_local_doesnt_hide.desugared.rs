fn f1() -> () {}
fn main() -> () {
    fn main__f2() -> () {
        crate::f1();
    }
    fn main__f3() -> () {}
    fn main__f4() -> () {
        main__f3();
    }
    let f1: bool;
    f1 = false;
    let f3: bool;
    f3 = false;
}
