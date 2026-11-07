//@ run
//@ expect-rejection: Invalid @ FormalityCheck
fn main() {
    let x: bool = false;
    let r: &bool = &x;
    let s: &mut bool = &mut x;
    print(*r);
}
