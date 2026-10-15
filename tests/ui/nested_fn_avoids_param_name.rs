// A nested function's new name avoids the name of a parameter.
fn bar(main__foo: bool) {}

fn main() {
    fn foo() {}
    foo();
    bar(true);
}
