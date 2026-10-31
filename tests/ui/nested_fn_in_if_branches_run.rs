//@ run
// Functions nested in the branches of an if expression.
// Expected output: true
fn main() {
    if true {
        fn foo() {
            print(true);
        }
        foo();
    } else {
        fn foo() {
            print(false);
        }
        foo();
    }
}
