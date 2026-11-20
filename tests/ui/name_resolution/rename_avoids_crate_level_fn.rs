// Unambiguous names for nested functions are chosen to avoid the names of crate-level items.
fn main__foo() {}

fn main() {
    fn foo() {}
    foo();
    main__foo();
}
