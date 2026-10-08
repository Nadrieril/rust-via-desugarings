fn main() {
    fn f1() {}
    fn f2() {
        fn f1() {}
        f1();
    }
    f1();
}
