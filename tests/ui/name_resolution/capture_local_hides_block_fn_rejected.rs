//@ expect-rejection: Invalid @ Desugaring
// A local introduced in an enclosing function shadows (inside a nested function) a function
// declared in that enclosing function.
fn main() {
    fn f1() {}
    let f1: bool = true;
    fn f2() {
        f1();
    }
}
