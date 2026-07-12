//@ known-failure
//@ run
fn main() {
    let x: bool = false;
    let r: &bool = &x;
    let s: &mut bool = &mut x;
    print(*r);
}
