//@ expect-rejection: Invalid @ Desugaring
// A `let` statement's `else` block can't see the local which the statement introduces.
fn main() {
    let x: bool = true else {
        print(x);
    };
}
