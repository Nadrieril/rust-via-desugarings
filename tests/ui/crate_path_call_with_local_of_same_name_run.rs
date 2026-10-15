//@ run
fn f1() {
    print(true);
}

fn main() {
    let f1: bool = false;
    crate::f1();
    print(f1);
}
