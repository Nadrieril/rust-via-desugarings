fn main() -> () {
    let pair: (bool, bool);
    pair = (false, true);
    pair.0 = true;
    let one: (bool,);
    one = (true,);
    one.0 = false;
    let nested: ((bool,), bool);
    nested = ((true,), false);
    nested.0.0 = false;
    let not_tuple: bool;
    not_tuple = true;
}
