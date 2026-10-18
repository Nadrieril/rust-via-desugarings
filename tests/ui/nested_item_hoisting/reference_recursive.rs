// A recursive call in a hoisted function is rewritten to find the function in its new location.
fn main() {
    fn foo() {
        foo();
    }
}
