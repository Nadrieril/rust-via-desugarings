// Inside a nested function, another nested function shadows a local introduced in the enclosing
// function.
fn main() {
    let f1: bool = false;
    fn f2() {
        fn f1() {}
        f1();
    }
}
