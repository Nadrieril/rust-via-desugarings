//@ run
// Locals and parameters shadow crate-level items of the same name.
// crate-relative path expressions still resolve to the items.
// Path expressions which resolve to crate-level items become crate-relative.
fn f1() {
    print(true);
}

fn f2(f1: bool) {
    print(f1);
}

fn main() {
    f1();
    let f1: bool = false;
    print(f1);
    crate::f1();
    f2(false);
}
