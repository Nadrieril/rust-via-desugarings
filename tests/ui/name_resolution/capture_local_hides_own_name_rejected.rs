//@ expect-rejection: Invalid @ Desugaring
// A local introduced in an enclosing function shadows (inside its own body) a nested function.
fn main() {
    let f1: bool = true;
    fn f1() {
        f1();
    }
}
