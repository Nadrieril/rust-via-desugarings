//@ known-failure
pub struct Wrapper<T: Clone>(T);

impl<T: Clone> Clone for Wrapper<T> {
    fn clone(&self) -> Self {
        Wrapper(self.0.clone())
    }
}

impl<T: Clone> Wrapper<T> {
    pub fn copy(&self) -> Self {
        self.clone()
    }
}
