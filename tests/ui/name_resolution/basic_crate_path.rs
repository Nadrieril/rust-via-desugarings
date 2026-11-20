// A crate-relative path expression resolves to the crate-level item, in particular in cases where a
// single-segment path expression would resolve to a nested function or a local with the same name.
fn foo() {}

fn main() {
    fn foo() {}
    fn bar() {
        crate::foo();
    }
    crate::foo();
    let foo: bool = false;
    crate::foo();
}
