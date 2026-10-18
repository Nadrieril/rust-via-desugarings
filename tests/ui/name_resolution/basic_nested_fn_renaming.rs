// Nested functions are given unambiguous names, and the path expressions which resolve to them are
// rewritten to match.
// A crate-level function with the same name as a nested function is unaffected.
fn foo() {}

fn bar() {
    foo();
}

fn main() {
    fn foo() {}
    {
        fn foo() {}
        fn baz() {
            foo();
        }
        foo();
    }
    foo();
}
