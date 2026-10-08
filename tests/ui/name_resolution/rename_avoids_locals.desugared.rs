fn bar(main__baz: bool) -> () {
    crate::print(place_to_value!(main__baz));
}
fn main() -> () {
    let main__foo: bool;
    main__foo = true;
    crate::main__foo_0();
    crate::main__baz_0();
    crate::print(place_to_value!(main__foo));
}
fn main__foo_0() -> () {}
fn main__baz_0() -> () {}
