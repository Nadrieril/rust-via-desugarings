//@ run
// A nested function's new name avoids the name of a local.
// Expected output: false true
fn main() {
    let main__foo: bool = true;
    fn foo() {
        print(false);
    }
    foo();
    print(main__foo);
}
