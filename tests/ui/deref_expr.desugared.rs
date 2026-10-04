fn f(foo: &&bool) -> () {
    *foo;
    **foo;
}
