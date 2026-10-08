//@ known-failure
fn main() {
    fn inner(x: bool, _: bool, x: bool) {}
}
