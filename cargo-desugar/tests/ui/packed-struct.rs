//@ known-failure
#[repr(packed(1))]
pub struct Packed(u8);

fn use_packed(x: Packed) -> Packed {
    x
}
