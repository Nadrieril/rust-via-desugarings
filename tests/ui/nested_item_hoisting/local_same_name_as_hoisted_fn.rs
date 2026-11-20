// A local with the same original name as a hoisted function in the same function isn't rewritten
// to a crate-relative path.
fn main() {
    fn f() -> bool {
        true
    }
    print(f());
    let f: bool = false;
    print(f);
}
