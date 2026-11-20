// A local shadows (in the following statements) an item which is declared in the same block,
// whether the item is declared before or after the `let` statement.
fn main() {
    fn f1() {}
    f1();
    let f1: bool = true;
    print(f1);
}

fn other() {
    f2();
    let f2: bool = true;
    fn f2() {}
    print(f2);
}
