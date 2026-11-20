//@ expect-rejection: Invalid @ Desugaring
// A local introduced in an enclosing function shadows (inside a nested function) a crate-level function.
fn f1() {}

fn main() {
    let f1: bool = true;
    fn f2() {
        f1();
    }
}
