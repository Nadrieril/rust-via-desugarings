// A function nested in an inner block shadows an outer block's local.
fn main() {
    let f1: bool = true;
    {
        fn f1() {
            print(false);
        }
        f1();
    }
    print(f1);
}
