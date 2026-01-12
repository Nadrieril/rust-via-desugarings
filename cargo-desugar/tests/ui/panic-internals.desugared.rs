#![feature(
    allocator_api,
    fmt_arguments_from_str,
    fmt_internals,
    libstd_sys_internals,
    panic_internals,
    print_internals,
    rt,
    try_trait_v2,
)]
#![allow(unused_braces, unused_parens, internal_features)]
pub fn explode() {
    {
        std::rt::begin_panic::<&str>("boom");
    };
}
