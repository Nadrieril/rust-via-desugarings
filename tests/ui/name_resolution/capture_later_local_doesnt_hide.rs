// A local declared after a nested function's use of a name doesn't shadow an item with the same
// name.
fn f1() {}

fn main() {
    fn f2() {
        f1();
    }
    fn f3() {}
    fn f4() {
        f3();
    }
    let f1: bool = false;
    let f3: bool = false;
}
