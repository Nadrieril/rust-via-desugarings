//@ run
fn main() {
    fn f1(x: &mut bool) {
        *x = true;
    }
    fn f2(x: &mut bool) {
        fn f3(x: &mut bool) {
            f1(x);
        }
        f3(x);
    }
    let x: bool = false;
    f2(&mut x);
    print(x);
}
