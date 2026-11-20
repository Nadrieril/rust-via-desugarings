// Hoisted functions are placed after the crate-level item which contained them, in the source
// order of their `fn` lines.
fn main() {
    {
        fn f1() {}
    }
    fn f2() {
        fn f3() {}
    }
    fn f4() {}
}

fn foo() {
    fn f5() {}
}
