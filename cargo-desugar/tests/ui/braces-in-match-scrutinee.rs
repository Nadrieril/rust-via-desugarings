pub fn main() {
    let _value = match Some(3) {
        None => panic!(),
        Some(_) => 5,
    };
}
