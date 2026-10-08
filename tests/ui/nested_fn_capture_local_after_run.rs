//@ run
// A local declared after a nested function doesn't hide a crate-level function from it.
// Expected output: true
fn f1() {
    print(true);
}

fn main() {
    fn f2() {
        f1();
    }
    f2();
    let f1: bool = false;
}
