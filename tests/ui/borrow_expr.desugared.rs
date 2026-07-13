fn f() -> () {
    &foo;
    &mut foo;
    &value_to_place!(&foo);
    &value_to_place!(&mut foo);
}
