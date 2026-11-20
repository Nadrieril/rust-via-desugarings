//@ run
// Locals introduced in a nested function, and the nested function's parameters, shadow locals from
// the enclosing function.
// (Locals aren't renamed yet; for now this is a "run" test so that the output shows that things are
// behaving as expected.)
fn main() {
    let x: bool = true;
    fn f1() {
        let x: bool = false;
        print(x);
    }
    fn f2(x: bool) {
        print(x);
    }
    f1();
    f2(false);
    print(x);
}
