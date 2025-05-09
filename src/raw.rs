use crate::util::{assert_unchecked, nonnull_slice, nonnull_subslice};
use core::ptr::NonNull;

mod pos;
pub(crate) use pos::*;

mod start;
pub(crate) use start::*;

mod lengths;
pub(crate) use lengths::*;

/// A slide over a raw `T` buffer.
#[repr(C)]
pub(crate) struct RawSlide<T> {
    start: Start<T>,
    end: Pos<T>,
    cursor: Pos<T>,
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new(slice: NonNull<[T]>, offset: usize) -> Self {
        unsafe {
            assert_unchecked!(
                offset <= slice.len(),
                "undefined behavior: `offset > slice.len()`"
            )
        };

        let start = Start(slice.cast());
        let end = unsafe { Pos::new(start, slice.len()) };
        let cursor = unsafe { Pos::new(start, offset) };

        Self { start, end, cursor }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    const fn lengths(&self) -> Lengths {
        let source = unsafe { self.end.offset_from_start(self.start) };
        let consumed = unsafe { self.cursor.offset_from_start(self.start) };
        let remaining = unsafe { self.end.offset_from(self.cursor) };

        unsafe { Lengths::new_unchecked(source, consumed, remaining) }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn slices(&self) -> [NonNull<[T]>; 3] {
        let source = nonnull_slice(self.start.0, self.lengths().source());
        let consumed = nonnull_slice(self.start.0, self.lengths().consumed());
        let remaining = nonnull_slice(
            unsafe { self.start.0.add(self.lengths().consumed()) },
            self.lengths().remaining(),
        );

        [source, consumed, remaining]
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn compiler_hints(&self) {
        let _ = self.slices();
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn source(&self) -> NonNull<[T]> {
        let [source, _, _] = self.slices();

        source
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn consumed(&self) -> NonNull<[T]> {
        let [_, consumed, _] = self.slices();

        consumed
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn remaining(&self) -> NonNull<[T]> {
        let [_, _, remaining] = self.slices();

        remaining
    }
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn offset(&self) -> usize {
        self.consumed().len()
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe {
            assert_unchecked!(
                offset <= self.source().len(),
                "undefined behavior: `offset > self.source().len()`"
            )
        };

        self.cursor = unsafe { Pos::new(self.start, offset) };
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn set_offset_checked(&mut self, offset: usize) -> bool {
        if offset <= self.source().len() {
            unsafe { self.set_offset_unchecked(offset) };

            true
        } else {
            false
        }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn set_offset(&mut self, offset: usize) {
        assert!(
            self.set_offset_checked(offset),
            "offset > self.source().len()"
        )
    }
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_unchecked(&self, n: usize) -> NonNull<[T]> {
        unsafe {
            assert_unchecked!(
                n <= self.remaining().len(),
                "undefined behavior: `n > self.remaining().len()`"
            )
        };

        nonnull_slice(self.remaining().cast(), n)
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_checked(&self, n: usize) -> Option<NonNull<[T]>> {
        if n <= self.remaining().len() {
            Some(unsafe { self.peek_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek(&self, n: usize) -> NonNull<[T]> {
        self.peek_checked(n).expect("n > self.remaining().len()")
    }
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_back_unchecked(&self, n: usize) -> NonNull<[T]> {
        unsafe {
            assert_unchecked!(
                n <= self.consumed().len(),
                "undefined behavior: `n > self.consumed().len()`"
            )
        };

        nonnull_slice(unsafe { self.remaining().cast().sub(n) }, n)
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_back_checked(&self, n: usize) -> Option<NonNull<[T]>> {
        if n <= self.consumed().len() {
            Some(unsafe { self.peek_back_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_back(&self, n: usize) -> NonNull<[T]> {
        self.peek_back_checked(n)
            .expect("n > self.consumed().len()")
    }
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn advance_unchecked(&mut self, n: usize) -> NonNull<[T]> {
        unsafe {
            assert_unchecked!(
                n <= self.remaining().len(),
                "undefined behavior: `n > self.remaining().len()`"
            )
        };

        self.cursor = unsafe { self.cursor.add(n) };

        unsafe { self.peek_back_unchecked(n) }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn advance_checked(&mut self, n: usize) -> Option<NonNull<[T]>> {
        if n <= self.remaining().len() {
            Some(unsafe { self.advance_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn advance(&mut self, n: usize) -> NonNull<[T]> {
        self.advance_checked(n).expect("n > self.remaining().len()")
    }
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn rewind_unchecked(&mut self, n: usize) -> NonNull<[T]> {
        unsafe {
            assert_unchecked!(
                n <= self.consumed().len(),
                "undefined behavior: `n > self.consumed().len()`"
            )
        };

        self.cursor = unsafe { self.cursor.sub(n) };

        unsafe { self.peek_unchecked(n) }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn rewind_checked(&mut self, n: usize) -> Option<NonNull<[T]>> {
        if n <= self.consumed().len() {
            Some(unsafe { self.rewind_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) const fn rewind(&mut self, n: usize) -> NonNull<[T]> {
        self.rewind_checked(n).expect("n > self.consumed().len()")
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
fn lengths(s: &mut RawSlide<u8>) -> bool {
    if s.remaining().len() >= 10 {
        s.advance(10);
        s.rewind(5);
        s.advance(2);

        true
    } else {
        false
    }
}
