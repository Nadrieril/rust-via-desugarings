//@ expect-rejection: Invalid @ Desugaring
// A nested function can't use a local introduced in an enclosing function.
// See discussion at https://github.com/rust-lang/rust/issues/33118 .
fn main() {
    let x: bool = true;
    fn f1() {
        print(x);
    }
}
