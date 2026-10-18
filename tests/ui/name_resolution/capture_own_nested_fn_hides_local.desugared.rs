fn main() -> () {
    let f1: bool;
    f1 = false;
    fn f2() -> () {
        fn f1() -> () {}
        f1();
    }
}
