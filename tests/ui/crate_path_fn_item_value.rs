fn foo() {}

fn main() {
    let x = crate::foo;
    let y = (crate::foo, { crate::main });
}
