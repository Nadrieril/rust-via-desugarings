//@ run
// A nested function can use its own local with the same name as an enclosing function's local.
// Expected output: false true
fn main() {
    let x: bool = true;
    fn f1() {
        let x: bool = false;
        print(x);
    }
    f1();
    print(x);
}
