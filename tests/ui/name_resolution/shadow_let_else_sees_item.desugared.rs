fn x() -> () {}
fn main() -> () {
    let x: bool = true else {
        crate::x();
    };
}
