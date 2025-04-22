use core::{marker::PhantomData, ptr::NonNull};

#[repr(C)]
pub(super) struct Zst<T> {
    pub(super) len: usize,
    pub(super) offset: usize,
    pub(super) _marker: PhantomData<*const T>,
}

impl<T> Zst<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn new_unchecked(slice: NonNull<[T]>, offset: usize) -> Zst<T> {
        Zst {
            len: slice.len(),
            offset,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn cast<U>(self) -> Zst<T> {
        Zst {
            len: self.len,
            offset: self.offset,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    #[must_use]
    pub const unsafe fn offset_len(&self, _start: NonNull<T>) -> (usize, usize) {
        (self.offset, self.len)
    }
}

impl<T> Clone for Zst<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Zst<T> {}
