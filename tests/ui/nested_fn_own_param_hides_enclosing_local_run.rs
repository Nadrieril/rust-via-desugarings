//@ run
// A nested function can use its own parameter with the same name as an enclosing function's local.
// Expected output: false true
fn main() {
    let x: bool = true;
    fn f1(x: bool) {
        print(x);
    }
    f1(false);
    print(x);
}
