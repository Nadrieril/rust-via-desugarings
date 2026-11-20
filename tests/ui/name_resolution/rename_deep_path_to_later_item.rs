// A path expression in a deeply nested function resolves to an item declared later in an outer
// block.
fn main() {
    fn outer() {
        fn inner() {
            later();
        }
        inner();
    }
    outer();
    fn later() {}
}
