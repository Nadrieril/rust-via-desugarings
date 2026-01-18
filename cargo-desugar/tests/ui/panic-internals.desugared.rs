#![feature(
    allocator_api,
    fmt_arguments_from_str,
    fmt_internals,
    fn_traits,
    libstd_sys_internals,
    never_type,
    panic_internals,
    print_internals,
    rt,
    try_trait_v2,
    try_trait_v2_residual,
    yeet_desugar_details,
    hint_must_use,
    temporary_niche_types
)]
#![allow(unused_braces, unused_parens, internal_features)]
pub fn explode() {
    {
        (std::rt::begin_panic::<&str>)("boom");
    };
}
