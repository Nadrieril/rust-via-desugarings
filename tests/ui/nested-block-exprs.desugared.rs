fn main() -> () {
    let value: bool;
    value = if {
        let x;
        x = true;
        x
    } {
        false
    } else {
        true
    };
    let value: &bool;
    value = {
        let y;
        y = &{
            let x;
            x = true;
            x
        };
        y
    };
}
