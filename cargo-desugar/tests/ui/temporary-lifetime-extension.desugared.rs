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
fn temp() -> (String, i32) {
    (<std::string::String as std::convert::From<&str>>::from("Hello"), 1)
}
fn main() {
    let a_8 = &(temp());
    let b_17 = [(&(temp()),)];
    let c_25 = &((temp()).0);
    let d_36 = &(*<std::string::String as std::ops::Index<
        std::ops::RangeFull,
    >>::index(&((temp()).0), std::ops::RangeFull {}));
    let e_54 = {
        let _ = 123;
        &(*<str as std::ops::Index<
            std::ops::RangeFull,
        >>::index(
            &(*<std::string::String as std::ops::Deref>::deref(&((temp()).0))),
            std::ops::RangeFull {},
        ))
    };
    let f_71 = if true { &(temp()) } else { &(*{ &(temp()) }) };
    let g_96 = match (true) {
        true => &(temp()),
        false => {
            &(*{
                let _ = 123;
                &(temp())
            })
        }
    };
    let h_110 = match (temp()) {
        owned_non_temporary_102 => &({ owned_non_temporary_102 }),
    };
    {
        std::io::_print({
            let args_119 = (
                &(a_8),
                &(b_17),
                &(c_25),
                &(d_36),
                &(e_54),
                &(f_71),
                &(g_96),
                &(h_110),
            );
            let args_212 = [
                core::fmt::rt::Argument::<
                    '_,
                >::new_debug::<&(std::string::String, i32)>(&(*(args_119).0)),
                core::fmt::rt::Argument::<
                    '_,
                >::new_debug::<[(&(std::string::String, i32),); 1]>(&(*(args_119).1)),
                core::fmt::rt::Argument::<
                    '_,
                >::new_debug::<&std::string::String>(&(*(args_119).2)),
                core::fmt::rt::Argument::<'_>::new_debug::<&str>(&(*(args_119).3)),
                core::fmt::rt::Argument::<'_>::new_debug::<&str>(&(*(args_119).4)),
                core::fmt::rt::Argument::<
                    '_,
                >::new_debug::<&(std::string::String, i32)>(&(*(args_119).5)),
                core::fmt::rt::Argument::<
                    '_,
                >::new_debug::<&(std::string::String, i32)>(&(*(args_119).6)),
                core::fmt::rt::Argument::<
                    '_,
                >::new_debug::<&(std::string::String, i32)>(&(*(args_119).7)),
            ];
            unsafe {
                std::fmt::Arguments::<
                    '_,
                >::new::<
                    25,
                    8,
                >(
                    &(*&[
                        192, 1, 32, 192, 1, 32, 192, 1, 32, 192, 1, 32, 192, 1, 32, 192,
                        1, 32, 192, 1, 32, 192, 1, 10, 0,
                    ]),
                    &(*&(args_212)),
                )
            }
        });
    };
}
