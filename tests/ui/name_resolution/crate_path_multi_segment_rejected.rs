//@ expect-rejection: Invalid @ Desugaring
// A crate-relative path expression with more than one segment after `crate::`.

// At present the only crate-level items are functions, so there's no way to have such a path
// expression which resolves.
fn foo() {}

fn main() {
    crate::foo::bar();
}
