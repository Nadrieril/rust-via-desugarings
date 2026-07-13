fn main() -> () {
    let value: bool;
    value = if true {
        false
    } else {
        true
    };
    if place_to_value!(value) {} else {}
}
