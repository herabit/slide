use core::ptr::NonNull;

/// A pointer to the start of a slide.
#[repr(transparent)]
pub(crate) struct Start<T>(pub(crate) NonNull<T>);

impl<T> Clone for Start<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Start<T> {}
