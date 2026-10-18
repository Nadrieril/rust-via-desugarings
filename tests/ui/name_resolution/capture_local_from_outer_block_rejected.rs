//@ expect-rejection: Invalid @ Desugaring
// A local introduced in an enclosing function shadows a crate-level function inside a function
// nested in an inner block.
fn f1() {}

fn main() {
    let f1: bool = true;
    {
        fn f2() {
            f1();
        }
    }
}
