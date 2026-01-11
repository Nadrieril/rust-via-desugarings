#![feature(
    allocator_api,
    fmt_arguments_from_str,
    fmt_internals,
    panic_internals,
    print_internals,
    try_trait_v2
)]
#![allow(unused_braces, unused_parens, internal_features)]

fn foo(
    (
        a_3,
        b_4,
        std::result::Result::<bool, bool>::Ok { 0: x_8 }
        | std::result::Result::<bool, bool>::Err { 0: x_8 },
    ): (u32, u32, Result<bool, bool>),
) -> u32 {
    a_3 + b_4 + (x_8 as _)
}
