fn assert_exists<T>() {}

pub fn check() {
    struct Inner;
    assert_exists::<Inner>();
    assert_exists::<&Inner>();
    assert_exists::<Box<Inner>>();
}
