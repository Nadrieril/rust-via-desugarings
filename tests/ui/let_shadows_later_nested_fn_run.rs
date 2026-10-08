//@ run
// A local shadows a nested function of the same name defined later in the same block.
// Expected output: true
fn main() {
    let f1: bool = true;
    fn f1() {
        print(false);
    }
    print(f1);
}
