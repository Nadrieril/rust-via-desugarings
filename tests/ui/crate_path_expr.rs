// Crate-relative path expressions.
fn foo() -> bool {
    true
}

fn main() {
    crate::foo();
    let x: bool = crate::foo();
    let y = (crate::foo, { crate::foo });
    fn bar() {
        crate::foo();
    }
}
