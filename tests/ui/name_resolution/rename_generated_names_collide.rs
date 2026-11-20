// Nested functions at different depths whose generated names would collide are given distinct
// unambiguous names.
fn main() {
    fn f__g() {}
    fn f() {
        fn g() {}
        g();
    }
    f__g();
}
