#![feature(
            allocator_api,
            fmt_arguments_from_str,
            fmt_internals,
            panic_internals,
            print_internals,
            try_trait_v2,
        )]

        #![allow(
            unused_braces,
            unused_parens,
            internal_features,
        )]

        #[attr = Repr {reprs: [ReprPacked(Align(1 bytes))]}]
struct Packed(u8);
fn use_packed(x_2: Packed) -> Packed {
x_2
}
