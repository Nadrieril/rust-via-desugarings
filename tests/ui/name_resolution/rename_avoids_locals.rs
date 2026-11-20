// Unambiguous names for nested functions are chosen to avoid the names of locals and parameters.
fn bar(main__baz: bool) {
    print(main__baz);
}

fn main() {
    let main__foo: bool = true;
    fn foo() {}
    fn baz() {}
    foo();
    baz();
    print(main__foo);
}
