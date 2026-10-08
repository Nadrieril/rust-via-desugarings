//@ run
fn main() {
    let x: bool = false;
    foo(&mut x);
    print(x);
    fn foo(x: &mut bool) {
        *x = true;
    }
}
