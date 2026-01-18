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
#![feature(min_specialization)]
trait MyFrom<T> {
    fn from(_: T) -> Self;
}
impl<T> MyFrom<T> for () {
    default fn from(_: T) -> Self {
        ()
    }
}
impl MyFrom<()> for () {
    fn from(_: ()) -> Self {
        ()
    }
}
