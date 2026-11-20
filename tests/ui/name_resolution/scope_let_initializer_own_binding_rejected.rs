//@ expect-rejection: Invalid @ Desugaring
// A `let` statement's initializer can't see the local which the statement introduces.
fn main() {
    let x: bool = x;
}
