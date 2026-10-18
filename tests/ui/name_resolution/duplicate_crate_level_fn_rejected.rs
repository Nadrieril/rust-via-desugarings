//@ expect-rejection: Invalid @ Desugaring
// Two crate-level items with the same name are rejected.
fn foo() {}

fn foo() {}

fn main() {}
