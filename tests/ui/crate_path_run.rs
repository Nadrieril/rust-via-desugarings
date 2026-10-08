//@ run
fn foo(x: &mut bool) {
    *x = true;
}

fn main() {
    let x: bool = false;
    crate::foo(&mut x);
    print(x);
}
