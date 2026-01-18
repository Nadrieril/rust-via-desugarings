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
pub fn show(v_2: i32) {
    {
        (std::io::_print)({
            let args_12 = (&(v_2),);
            let args_28 = [
                (core::fmt::rt::Argument::<'_>::new_display::<i32>)(&(*(args_12).0)),
            ];
            unsafe {
                (std::fmt::Arguments::<
                    '_,
                >::new::<4, 1>)(&(*&[192, 1, 10, 0]), &(*&(args_28)))
            }
        });
    };
}
