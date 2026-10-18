// A `let` statement's initializer sees an item which the local introduced by the statement will shadow.
fn f1() -> bool {
    true
}

fn main() {
    let f1: bool = f1();
    print(f1);
}
