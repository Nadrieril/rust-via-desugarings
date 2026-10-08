fn main() {
    fn foo() {}
    {
        fn foo() {}
        fn bar() {
            foo();
        }
        foo();
    }
    foo();
}
