//@ run
fn main() {
    let x: bool = false;
    let r: &mut bool = &mut x;
    *r = true;
    print(x);
}
