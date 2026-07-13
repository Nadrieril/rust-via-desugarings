fn main() -> () {
    let value: bool;
    value = if {
        let x;
        x = true;
        place_to_value!(x)
    } {
        false
    } else {
        true
    };
    let value: &bool;
    value = {
        let y;
        y = &value_to_place!({
            let x;
            x = true;
            place_to_value!(x)
        });
        place_to_value!(y)
    };
}
