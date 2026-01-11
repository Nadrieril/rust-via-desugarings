#![feature(
    allocator_api,
    fmt_arguments_from_str,
    fmt_internals,
    panic_internals,
    print_internals,
    try_trait_v2
)]
#![allow(unused_braces, unused_parens, internal_features)]
#![feature(min_specialization)]
trait MyFrom<T> {
    fn from(_: T) -> Self;
}
impl<T> MyFrom<T> for () {
    fn from(_: T) -> Self {
        ()
    }
}
impl MyFrom<()> for () {
    fn from(_: ()) -> Self {
        ()
    }
}
