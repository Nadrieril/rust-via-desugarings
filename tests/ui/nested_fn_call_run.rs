//@ run
fn main() {
    fn foo(x: &mut bool) {
        *x = true;
    }
    let x: bool = false;
    foo(&mut x);
    print(x);
}
