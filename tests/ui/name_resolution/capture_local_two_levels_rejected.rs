//@ expect-rejection: Invalid @ Desugaring
// A local introduced in a doubly-enclosing function shadows a crate-level function.
fn f1() {}

fn main() {
    let f1: bool = true;
    fn f2() {
        fn f3() {
            f1();
        }
    }
}
