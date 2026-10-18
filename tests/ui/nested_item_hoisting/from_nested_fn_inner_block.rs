// A function nested in an inner block of a nested function is hoisted.
fn main() {
    fn f1() {
        {
            fn f2() {}
            f2();
        }
    }
}
