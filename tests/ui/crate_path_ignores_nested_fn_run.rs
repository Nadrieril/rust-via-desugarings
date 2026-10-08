//@ run
fn foo(x: &mut bool) {
    *x = true;
}

fn main() {
    fn foo(x: &mut bool) {}
    let x: bool = false;
    crate::foo(&mut x);
    print(x);
}
