//@ expect-rejection: Invalid @ Desugaring
// A nested function's parameters can't bind the same name twice.
fn main() {
    fn nested(a: bool, a: bool) {}
}
