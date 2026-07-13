fn main() -> () {
    let x: bool;
    x = false;
    let r: &mut bool;
    r = &mut x;
    *r = true;
    print(x);
}
