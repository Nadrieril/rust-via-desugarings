// A function nested in a nested function shadows an outer nested function which has the same name.
fn main() {
    fn f1() {}
    fn f2() {
        fn f1() {}
        f1();
    }
    f1();
}
