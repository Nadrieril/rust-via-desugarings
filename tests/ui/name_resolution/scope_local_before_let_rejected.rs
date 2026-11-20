//@ expect-rejection: Invalid @ Desugaring
// A local isn't visible before the `let` statement which introduces it.
fn main() {
    print(x);
    let x: bool = true;
}
