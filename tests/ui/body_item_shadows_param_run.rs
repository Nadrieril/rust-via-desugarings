//@ run
fn f2(f1: bool) {
    fn f1() {
        print(true);
    }
    f1();
}

fn main() {
    f2(false);
}
