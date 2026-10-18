// A local introduced in an enclosing function doesn't shadow a crate-level function after its block is closed.
fn f1() {}

fn main() {
    if true {
        let f1: bool = false;
        print(f1);
    }
    fn f2() {
        f1();
    }
}
