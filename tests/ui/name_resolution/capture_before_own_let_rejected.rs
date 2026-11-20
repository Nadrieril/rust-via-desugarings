//@ expect-rejection: Invalid @ Desugaring
// A nested function can't use a local introduced in an enclosing function.
// Case where the nested function later introduces a local with the same name.
fn main() {
    let x: bool = true;
    fn f1() {
        print(x);
        let x: bool = false;
    }
}
