//@ run
fn main() {
    fn f1() {
        print(true);
    }
    fn f2() {
        f1();
    }
    let f1: bool = false;
    f2();
}
