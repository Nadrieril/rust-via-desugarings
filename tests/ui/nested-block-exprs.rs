fn main() {
    let value: bool = if {
        let x = true;
        x
    } {
        false
    } else {
        true
    };

    let value: &bool = {
        let y = &{
            let x = true;
            x
        };
        y
    };
}
