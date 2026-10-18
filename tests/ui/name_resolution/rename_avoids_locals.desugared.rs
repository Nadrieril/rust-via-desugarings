fn bar(main__baz: bool) -> () {
    crate::print(place_to_value!(main__baz));
}
fn main() -> () {
    let main__foo: bool;
    main__foo = true;
    fn main__foo_0() -> () {}
    fn main__baz_0() -> () {}
    main__foo_0();
    main__baz_0();
    crate::print(place_to_value!(main__foo));
}
