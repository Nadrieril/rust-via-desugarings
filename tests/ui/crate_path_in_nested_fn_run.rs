//@ run
fn foo(x: &mut bool) {
    *x = true;
}

fn main() {
    fn bar(x: &mut bool) {
        crate::foo(x);
    }
    let x: bool = false;
    bar(&mut x);
    print(x);
}
