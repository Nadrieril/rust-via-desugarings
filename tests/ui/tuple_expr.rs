fn main() {
    let pair: (bool, bool) = (false, true);
    pair.0 = true;

    let one: (bool,) = (true,);
    one.0 = false;

    let nested: ((bool,), bool) = ((true,), false);
    nested.0.0 = false;

    let not_tuple: bool = (true);
}
