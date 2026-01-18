
            #![feature(allocator_api,fmt_arguments_from_str,fmt_internals,fn_traits,libstd_sys_internals,never_type,panic_internals,print_internals,rt,try_trait_v2,try_trait_v2_residual,yeet_desugar_details,hint_must_use,temporary_niche_types)]

            #![allow(
                unused_braces,
                unused_parens,
                internal_features,
            )]
pub fn takes<impl Iterator<Item = u32>>(_: impl Iterator<Item = u32>) where
    impl Iterator<Item = u32>: Iterator<Item = u32> {
}
pub fn call<I>(iter_2: I) where I: Iterator<Item =
    u32> {
(takes::<I>)(iter_2);
}
