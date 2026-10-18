// A `let` statement's initializer sees an outer local which the local introduced by the statement will shadow.
fn main() {
    let x: bool = true;
    {
        let x: bool = x;
        print(x);
    }
}
