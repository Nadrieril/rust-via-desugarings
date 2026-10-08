//@ run
// An enclosing function's local doesn't hide a crate-level function once its block has closed.
// Expected output: false true
fn f1() {
    print(true);
}

fn main() {
    if true {
        let f1: bool = false;
        print(f1);
    }
    fn f2() {
        f1();
    }
    f2();
}
