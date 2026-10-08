fn foo() {}

fn bar() {
    foo();
}

fn main() {
    fn foo() {}
    fn f1() {
        foo();
    }
    foo();
}
