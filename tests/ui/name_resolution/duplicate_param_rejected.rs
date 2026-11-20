//@ expect-rejection: Invalid @ Desugaring
// A function's parameters can't bind the same name twice (but wildcards can be repeated).
fn foo(a: bool, _: bool, _: bool, a: bool) {}

fn main() {}
