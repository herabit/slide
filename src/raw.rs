use core::ptr::NonNull;

use crate::{macros::assert_unchecked, pos::Pos};

#[repr(C)]
pub(crate) struct RawSlide<T> {
    start: NonNull<T>,
    cursor: Pos<T>,
    end: Pos<T>,
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_parts(slice: NonNull<[T]>, offset: usize) -> Self {
        unsafe { assert_unchecked!(offset <= slice.len(), "`offset > slice.len()`") };

        Self {
            start: slice.cast(),
            cursor: unsafe { Pos::with_offset(slice.cast(), offset) },
            end: unsafe { Pos::with_offset(slice.cast(), slice.len()) },
        }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn compiler_hints(&self) {
        let end = unsafe { self.end.get(self.start) };
        let cursor = unsafe { self.cursor.get(self.start) };

        let source_len = unsafe { end.to_offset(self.start) };
        let consumed_len = unsafe { cursor.to_offset(self.start) };
        let remaining_len = unsafe { end.offset_from(cursor) };

        // Assert that `consumed_len <= source_len` and `remaining_len <= source_len`.
        //
        // You would think tht these two facts would be obvious to LLVM with the assertions
        // after these two, but apparently not in the case of ZSTs.
        unsafe {
            assert_unchecked!(
                consumed_len <= source_len,
                "`self.consumed().len() > self.source().len()`"
            );

            assert_unchecked!(
                remaining_len <= source_len,
                "`self.remaining().len() > self.source().len()`"
            );
        }

        // Assert that `consumed_len + remaining_len == source_len`.
        //
        // We also do subtractions here as LLVM for some reason fails to optimize properly
        // without further assertions.
        unsafe {
            assert_unchecked!(
                source_len.unchecked_sub(remaining_len) == consumed_len,
                "`self.source().len() - self.remaining().len() != self.consumed().len()`"
            );

            assert_unchecked!(
                source_len.unchecked_sub(consumed_len) == remaining_len,
                "`self.source().len() - self.consumed().len() != self.remaining().len()`"
            );
            assert_unchecked!(
                consumed_len.unchecked_add(remaining_len) == source_len,
                "`self.consumed().len() + self.remaining().len() != self.source().len()`"
            );

            assert_unchecked!(
                remaining_len.unchecked_add(consumed_len) == source_len,
                "`self.remaining().len() + self.consumed().len() != self.source().len()`"
            );
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn cursor(&self) -> Pos<T> {
        self.compiler_hints();

        unsafe { self.cursor.get(self.start) }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn end(&self) -> Pos<T> {
        self.compiler_hints();

        unsafe { self.end.get(self.start) }
    }
}

impl<T> Clone for RawSlide<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RawSlide<T> {}

#[unsafe(no_mangle)]
fn lol(s: &RawSlide<()>) -> bool {
    unsafe { s.cursor().to_offset(s.start) <= s.end().to_offset(s.start) }
}
