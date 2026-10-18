// A function nested in a block expression is hoisted.
fn main() {
    let x: bool = {
        fn f() -> bool {
            true
        }
        f()
    };
}
