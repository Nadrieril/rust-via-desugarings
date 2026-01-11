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

        pub fn main() {
let value_27 = match std::option::Option::<i32>::Some{ 0: 3 } {
std::option::Option::<i32>::None => {
std::rt::begin_panic::<&str>("explicit panic");
},
std::option::Option::<i32>::Some { 0: _ } => 5,
};
{
std::io::_print({
let args_36 = (&value_27,);
let args_52 = [<core::fmt::rt::Argument<'_>>::new_display::<i32>(&*args_36.0)];
unsafe {
<std::fmt::Arguments<'_>>::new::<4, 1>(&*&[192, 1, 10, 0], &*&args_52)
}
});
};
}
