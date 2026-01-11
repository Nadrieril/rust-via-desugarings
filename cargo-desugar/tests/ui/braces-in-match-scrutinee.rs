//@ known-failure
pub fn main() {
    let value = match Some(3) {
        None => panic!("explicit panic"),
        Some(_) => 5,
    };
    println!("{value}");
}
