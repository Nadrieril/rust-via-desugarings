//@ known-failure
pub fn takes(_: impl Iterator<Item = u32>) {}

pub fn call<I: Iterator<Item = u32>>(iter: I) {
    takes(iter);
}
