fn main__foo() {}

fn main() {
    fn foo() {}
    foo();
    main__foo();
}
