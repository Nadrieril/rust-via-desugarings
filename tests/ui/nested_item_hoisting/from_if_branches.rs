// Functions nested in the branches of an if expression are hoisted.
fn main() {
    if true {
        fn foo() {}
        foo();
    } else {
        fn foo() {}
        foo();
    }
}
