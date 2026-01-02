# Enum Discriminant Access

To read the discriminant of an enum value today, one must use `std::mem::discriminant(&place)`. This
unfortunately requires a `&`-borrow, which may introduce UB in unsafe contexts.
Moreover there's no stable way to get a discriminant value without having an enum value with that
discriminant around.

To solve this, I propose to combine ideas from two existing RFCs:
- from [RFC 3607](https://github.com/rust-lang/rfcs/pull/3607) we add a new place expression
  `$place.enum#discriminant` evaluates to the discriminant value at that place;
- from [RFC 3727](https://github.com/rust-lang/rfcs/pull/3727), we add `discriminant_of!($enum_type,
  $variant_name)` that returns the discriminant value for that variant.

Using both, we have that `matches!($place, Some(_))` is equivalent to `$place.enum#discriminant ==
discriminant_of!(Option<_>, Some)`.

Those two will also come useful for [Phased Initialization](phased-initialization.md).
