#![feature(
    allocator_api,
    fmt_arguments_from_str,
    fmt_internals,
    panic_internals,
    print_internals,
    try_trait_v2
)]
#![allow(unused_braces, unused_parens, internal_features)]

struct Wrapper<T>(T)
where
    T: Clone;
impl<T> Clone for Wrapper<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Wrapper::<T> {
            0: <T as std::clone::Clone>::clone(&*self_2.0),
        }
    }
}
impl<T> Wrapper<T>
where
    T: Clone,
{
    fn copy(&self) -> Self {
        <Wrapper<T> as std::clone::Clone>::clone(&*self_2)
    }
}
