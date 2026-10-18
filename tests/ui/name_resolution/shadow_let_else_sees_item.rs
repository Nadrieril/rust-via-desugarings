// A `let` statement's `else` block sees an item which the local introduced by the statement will
// shadow.
fn x() {}

fn main() {
    let x: bool = true else {
        x();
        // `return` isn't in the grammar yet.
        // return;
    };
}
