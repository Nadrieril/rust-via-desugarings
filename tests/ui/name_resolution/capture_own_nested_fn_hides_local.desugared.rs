fn main() -> () {
    let f1: bool;
    f1 = false;
    fn main__f2() -> () {
        fn main__f2__f1() -> () {}
        main__f2__f1();
    }
}
