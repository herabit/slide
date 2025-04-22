use core::ptr::NonNull;

use crate::util;

/// Helper struct for sliding over slices of types that are not
/// zero sized.
#[repr(C)]
pub(super) struct NonZst<T> {
    pub(super) end: NonNull<T>,
    pub(super) cursor: NonNull<T>,
}

impl<T> NonZst<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn new_unchecked(slice: NonNull<[T]>, offset: usize) -> NonZst<T> {
        let start = slice.cast::<T>();
        let end = unsafe { start.add(slice.len()) };
        let cursor = unsafe { start.add(offset) };

        NonZst { end, cursor }
    }

    #[inline(always)]
    #[must_use]
    pub const unsafe fn cast<U>(self) -> NonZst<U> {
        NonZst {
            end: self.end.cast(),
            cursor: self.cursor.cast(),
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn offset_len(&self, start: NonNull<T>) -> (usize, usize) {
        let offset = unsafe { util::unchecked_sub(self.cursor.as_ptr(), start.as_ptr()) };
        let len = unsafe { util::unchecked_sub(self.end.as_ptr(), start.as_ptr()) };

        (offset, len)
    }
}

impl<T> Clone for NonZst<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NonZst<T> {}
