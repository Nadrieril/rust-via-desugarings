fn foo() -> () {}
fn bar() -> () {
    crate::foo();
}
fn main() -> () {
    fn main__foo() -> () {}
    {
        fn main__foo_0() -> () {}
        fn main__baz() -> () {
            main__foo_0();
        }
        main__foo_0();
    }
    main__foo();
}
