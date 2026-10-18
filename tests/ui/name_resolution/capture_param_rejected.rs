//@ expect-rejection: Invalid @ Desugaring
// A nested function can't use a parameter of an enclosing function.
fn f1(x: bool) {
    fn f2() {
        print(x);
    }
}
