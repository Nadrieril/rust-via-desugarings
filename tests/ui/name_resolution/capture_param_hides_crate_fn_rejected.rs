//@ expect-rejection: Invalid @ Desugaring
// A parameter of an enclosing function shadows (inside a nested function) a crate-level function.
fn f1() {}

fn f2(f1: bool) {
    fn f3() {
        f1();
    }
}

fn main() {}
