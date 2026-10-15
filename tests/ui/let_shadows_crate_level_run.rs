//@ run
fn f1() {
    print(true);
}

fn main() {
    f1();
    let f1: bool = false;
    print(f1);
}
