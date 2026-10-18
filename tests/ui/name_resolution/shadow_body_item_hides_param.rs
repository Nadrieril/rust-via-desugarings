// An item in a function's body shadows that function's parameters.
fn f2(f1: bool) {
    fn f1() {}
    f1();
}
