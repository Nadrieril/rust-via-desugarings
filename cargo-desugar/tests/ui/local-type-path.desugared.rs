#![feature(
    allocator_api,
    fmt_arguments_from_str,
    fmt_internals,
    panic_internals,
    print_internals,
    try_trait_v2
)]
#![allow(unused_braces, unused_parens, internal_features)]

fn assert_exists<T>() {}
pub fn check() {
    struct Inner;
    {
        assert_exists::<check::Inner>();
    }
}
