//@ known-failure
fn main() {
    let x: bool = true;
    fn f1() {
        print(x);
    }
    f1();
}
