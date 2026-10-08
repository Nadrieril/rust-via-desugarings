//@ run
fn f1() {}

fn f2(f1: bool) {
    print(f1);
}

fn main() {
    f2(true);
}
