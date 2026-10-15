//@ known-failure
fn main() {
    fn foo() {
        crate::bar();
    }
}
