//@ run
// A nested function's own nested function hides an enclosing function's local.
// Expected output: true
fn main() {
    let f1: bool = false;
    fn f2() {
        fn f1() {
            print(true);
        }
        f1();
    }
    f2();
}
