//@ run
// Nested functions are moved to crate level, and path expressions which resolved to them become
// crate-relative.
// Calls between nested functions work (including calls from deeper levels).
//
// NB rustc rejects this program because `x` isn't declared `mut`.
// (The grammar doesn't have `mut` bindings yet; the runners treat every local as mutable.)
fn main() {
    fn f1(x: &mut bool) {
        *x = true;
    }
    fn f2(x: &mut bool) {
        fn f3(x: &mut bool) {
            f1(x);
        }
        f3(x);
    }
    let x: bool = false;
    print(x);
    f2(&mut x);
    print(x);
}
