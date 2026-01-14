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
pub fn main() {
    let _value_26 = match (std::option::Option::<i32>::Some {
        0: 3,
    }) {
        std::option::Option::<i32>::None => {
            ((std::rt::begin_panic::<&str>("explicit panic")) as i32)
        }
        std::option::Option::<i32>::Some { 0: _ } => 5,
    };
}
