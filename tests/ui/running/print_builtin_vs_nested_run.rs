//@ run
// The runners provide `print` as if it were a crate-level function.
// A nested function named `print` shadows it.
// A crate-relative path expression still resolves to it.
fn main() {
    fn print(x: bool) {}
    print(false);
    crate::print(true);
}
