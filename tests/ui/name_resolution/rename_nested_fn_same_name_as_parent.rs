// A nested function can have the same name as the function it's nested in, and renaming keeps the
// two apart.
fn main() {
    fn twin() {
        fn twin() {}
        twin();
    }
    twin();
}
