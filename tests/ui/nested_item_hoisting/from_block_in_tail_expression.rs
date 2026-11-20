// A function nested in a block inside a block's tail expression is hoisted.
fn main() {
    print({
        fn f() -> bool {
            true
        }
        f()
    })
}
