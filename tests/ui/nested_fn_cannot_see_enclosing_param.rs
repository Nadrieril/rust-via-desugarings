//@ known-failure
fn f1(x: bool) {
    fn f2() {
        print(x);
    }
    f2();
}

fn main() {
    f1(true);
}
