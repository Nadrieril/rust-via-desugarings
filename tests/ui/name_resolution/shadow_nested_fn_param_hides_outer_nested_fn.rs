// A parameter of a nested function parameter shadows an outer nested function which has the same name.
fn main() {
    fn f1() {}
    fn f2(f1: bool) {
        print(f1);
    }
}
