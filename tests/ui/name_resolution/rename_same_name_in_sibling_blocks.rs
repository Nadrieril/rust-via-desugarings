// Nested functions with the same name in sibling blocks are given distinct unambiguous names.
fn main() {
    {
        fn foo() {}
        foo();
    }
    {
        fn foo() {}
        foo();
    }
}
