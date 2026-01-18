#![feature(min_specialization)]
trait MyFrom<T> {
    fn from(_: T) -> Self;
}

impl<T> MyFrom<T> for () {
    default fn from(_: T) -> Self {
        ()
    }
}

impl MyFrom<()> for () {
    fn from(_: ()) -> Self {
        ()
    }
}
