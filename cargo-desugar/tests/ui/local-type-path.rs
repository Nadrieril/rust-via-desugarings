//@ known-failure
fn assert_exists<T>() {}

pub fn check() {
    struct Inner;
    assert_exists::<Inner>();
}
