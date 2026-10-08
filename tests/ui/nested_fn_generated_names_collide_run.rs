//@ run
// Two nested functions whose generated names would collide get distinct names.
// Expected output: true false
fn main() {
    fn f__g() {
        print(true);
    }
    fn f() {
        fn g() {
            print(false);
        }
        g();
    }
    f__g();
    f();
}
