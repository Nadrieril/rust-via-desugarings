//@ run
fn main() {
    fn f1() {}
    fn f2(f1: bool) {
        print(f1);
    }
    f2(true);
}
