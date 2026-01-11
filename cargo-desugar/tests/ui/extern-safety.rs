//@ known-failure
extern "C" {
    fn ffi();
}

pub fn call() {
    unsafe { ffi(); }
}
