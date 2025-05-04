#![allow(dead_code)]

use core::ptr::NonNull;

use crate::util::{self, assert_unchecked};

mod non_zst;
mod zst;

/// A raw slide over a buffer.
///
/// # Safety
///
/// Its bits must be a valid [`zst::Slide`] if `T` is zero sized,
/// or a valid [`non_zst::Slide`] if `T` is not zero sized.
#[repr(C)]
pub(crate) struct RawSlide<T> {
    _start: NonNull<T>,
    _data: [*const T; 2],
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    const unsafe fn new(slice: NonNull<[T]>, offset: usize) -> Option<Self> {
        if offset <= slice.len() {
            Some(match size_of::<T>() {
                0 => unsafe { zst::Slide::new(slice, offset) }.into_raw(),
                1.. => unsafe { non_zst::Slide::new(slice, offset) }.into_raw(),
            })
        } else {
            None
        }
    }

    #[inline(always)]
    #[must_use]
    pub(crate) const fn from_slice_offset(slice: &[T], offset: usize) -> Option<Self> {
        unsafe { Self::new(util::nonnull_from_ref(slice), offset) }
    }

    #[inline(always)]
    #[must_use]
    pub(crate) const fn from_slice_mut_offset(slice: &mut [T], offset: usize) -> Option<Self> {
        unsafe { Self::new(util::nonnull_from_mut(slice), offset) }
    }

    #[inline(always)]
    #[must_use]
    pub(crate) const fn from_slice(slice: &[T]) -> Self {
        Self::from_slice_offset(slice, 0).unwrap()
    }

    #[inline(always)]
    #[must_use]
    pub(crate) const fn from_slice_mut(slice: &mut [T]) -> Self {
        Self::from_slice_mut_offset(slice, 0).unwrap()
    }
}

impl<T> RawSlide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    const fn as_zst(&self) -> &zst::Slide<T> {
        let x = zst::Slide::from_raw_ref(self);
        x.compiler_hints();
        x
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    const fn as_non_zst(&self) -> &non_zst::Slide<T> {
        let x = non_zst::Slide::from_raw_ref(self);
        x.compiler_hints();
        x
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    const fn as_zst_mut(&mut self) -> &mut zst::Slide<T> {
        let x = zst::Slide::from_raw_mut(self);
        x.compiler_hints();
        x
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    const fn as_non_zst_mut(&mut self) -> &mut non_zst::Slide<T> {
        let x = non_zst::Slide::from_raw_mut(self);
        x.compiler_hints();
        x
    }

    #[inline(always)]
    #[must_use]
    const fn borrow(&self) -> SlideRef<'_, T> {
        match size_of::<T>() {
            0 => SlideRef::Zst(self.as_zst()),
            1.. => SlideRef::NonZst(self.as_non_zst()),
        }
    }

    #[inline(always)]
    #[must_use]
    const fn borrow_mut(&mut self) -> SlideMut<'_, T> {
        match size_of::<T>() {
            0 => SlideMut::Zst(self.as_zst_mut()),
            1.. => SlideMut::NonZst(self.as_non_zst_mut()),
        }
    }
}

impl<T> RawSlide<T> {
    /// Provide hints to the compiler.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn compiler_hints(&self) {
        match self.borrow() {
            SlideRef::Zst(slide) => slide.compiler_hints(),
            SlideRef::NonZst(slide) => slide.compiler_hints(),
        }
    }

    /// Return the entire source buffer.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn source(&self) -> NonNull<[T]> {
        match self.borrow() {
            SlideRef::Zst(slide) => slide.source(),
            SlideRef::NonZst(slide) => slide.source(),
        }
    }

    /// Return the consumed buffer.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn consumed(&self) -> NonNull<[T]> {
        match self.borrow() {
            SlideRef::Zst(slide) => slide.consumed(),
            SlideRef::NonZst(slide) => slide.consumed(),
        }
    }

    /// Return the remaining buffer.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn remaining(&self) -> NonNull<[T]> {
        match self.borrow() {
            SlideRef::Zst(slide) => slide.remaining(),
            SlideRef::NonZst(slide) => slide.remaining(),
        }
    }

    /// Return whether the slide is done.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn is_done(&self) -> bool {
        self.remaining().len() == 0
    }
}

impl<T> RawSlide<T> {
    /// Return the current offset in the slide.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn offset(&self) -> usize {
        match self.borrow() {
            SlideRef::Zst(slide) => slide.offset(),
            SlideRef::NonZst(slide) => slide.offset(),
        }
    }

    /// Set the current offset in the slide without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `offset <= self.source().len()`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe {
            assert_unchecked!(
                offset <= self.source().len(),
                "undefined behavior: `offset > self.source().len()`"
            )
        };

        // SAFETY: The caller ensures that `offset <= self.source().len()`.
        match self.borrow_mut() {
            SlideMut::Zst(slide) => unsafe { slide.set_offset_unchecked(offset) },
            SlideMut::NonZst(slide) => unsafe { slide.set_offset_unchecked(offset) },
        }
    }

    /// Set the current offset in the slide if `offset <= self.source().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if the offset was not set.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn set_offset_checked(&mut self, offset: usize) -> Option<()> {
        if offset <= self.source().len() {
            Some(unsafe { self.set_offset_unchecked(offset) })
        } else {
            None
        }
    }

    /// Set the current offset in the slide.
    ///
    /// # Panics
    ///
    /// Panics if `offset > self.source().len()`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn set_offset(&mut self, offset: usize) {
        self.set_offset_checked(offset)
            .expect("offset > self.source().len()")
    }
}

impl<T> RawSlide<T> {
    /// Advance the slide `n` elements without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.remaining().len()`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn advance_unchecked(&mut self, n: usize) {
        unsafe {
            assert_unchecked!(
                n <= self.remaining().len(),
                "undefined behavior: `n > self.remaining().len()`"
            )
        };

        // SAFETY: The caller ensures this is allowed.
        match self.borrow_mut() {
            SlideMut::Zst(slide) => unsafe { slide.advance_unchecked(n) },
            SlideMut::NonZst(slide) => unsafe { slide.advance_unchecked(n) },
        }
    }

    /// Advance the slide `n` elements if `n <= self.remaining().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if the slide was not advanced.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn advance_checked(&mut self, n: usize) -> Option<()> {
        if n <= self.remaining().len() {
            // SAFETY: We just checked that it is allowed.
            Some(unsafe { self.advance_unchecked(n) })
        } else {
            None
        }
    }

    /// Rewind the slide `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if the slide was not rewound.

    /// Advance the slide `n` elements.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.remaining().len()`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn advance(&mut self, n: usize) {
        self.advance_checked(n).expect("n > self.remaining().len()")
    }
}

impl<T> RawSlide<T> {
    /// Rewind the slide `n` elements without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.consumed().len()`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn rewind_unchecked(&mut self, n: usize) {
        unsafe {
            assert_unchecked!(
                n <= self.consumed().len(),
                "undefined behavior: `n > self.consumed().len()`"
            )
        };

        // SAFETY: The caller ensures this is allowed.
        match self.borrow_mut() {
            SlideMut::Zst(slide) => unsafe { slide.rewind_unchecked(n) },
            SlideMut::NonZst(slide) => unsafe { slide.rewind_unchecked(n) },
        }
    }

    /// Rewind the slide `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.consumed().len()`.
    #[inline(always)]
    #[track_caller]
    #[must_use]
    pub(crate) const fn rewind_checked(&mut self, n: usize) -> Option<()> {
        if n <= self.consumed().len() {
            Some(unsafe { self.rewind_unchecked(n) })
        } else {
            None
        }
    }

    /// Rewind the slide `n` elements.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.consumed().len()`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn rewind(&mut self, n: usize) {
        self.rewind_checked(n).expect("n > self.consumed().len()")
    }
}

impl<T> RawSlide<T> {
    /// Peek the next `n` elements without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `n <= self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_slice_unchecked(&self, n: usize) -> NonNull<[T]> {
        unsafe {
            assert_unchecked!(
                n <= self.remaining().len(),
                "undefined behavior: `n > self.remaining().len()`"
            )
        };

        match self.borrow() {
            SlideRef::Zst(slide) => unsafe { slide.peek_slice_unchecked(n) },
            SlideRef::NonZst(slide) => unsafe { slide.peek_slice_unchecked(n) },
        }
    }

    /// Peek the next `N` elements as an array without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `N <= self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_array_unchecked<const N: usize>(&self) -> NonNull<[T; N]> {
        unsafe {
            assert_unchecked!(
                N <= self.remaining().len(),
                "undefined behavior: `N > self.remaining().len()`"
            )
        };

        unsafe { self.peek_slice_unchecked(N) }.cast()
    }

    /// Peek the next element without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self.remaining().len() != 0`
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_unchecked(&self) -> NonNull<T> {
        unsafe {
            assert_unchecked!(
                self.remaining().len() != 0,
                "undefined behavior: `self.remaining().len() == 0`"
            )
        };

        unsafe { self.peek_slice_unchecked(1) }.cast()
    }

    /// Peek the next `n` elements if `n <= self.remaining().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_slice_checked(&self, n: usize) -> Option<NonNull<[T]>> {
        if n <= self.remaining().len() {
            Some(unsafe { self.peek_slice_unchecked(n) })
        } else {
            None
        }
    }

    /// Peek the next `N` elements as an array if `N <= self.remaining().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `N > self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_array_checked<const N: usize>(&self) -> Option<NonNull<[T; N]>> {
        match self.peek_slice_checked(N) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    /// Peek the next element if `self.remaining().len() != 0`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `self.remaining().len() == 0`
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_checked(&self) -> Option<NonNull<T>> {
        match self.peek_slice_checked(1) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    /// Peek the next `n` elements.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_slice(&self, n: usize) -> NonNull<[T]> {
        self.peek_slice_checked(n)
            .expect("n > self.remaining().len()")
    }

    /// Peek the next `N` elements as an array.
    ///
    /// # Panics
    ///
    /// Panics if `N > self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_array<const N: usize>(&self) -> NonNull<[T; N]> {
        self.peek_array_checked::<N>()
            .expect("N > self.remaining().len()")
    }

    /// Peek the next element.
    ///
    /// # Panics
    ///
    /// Panics if `self.remaining().len() == 0`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek(&self) -> NonNull<T> {
        self.peek_checked().expect("self.remaining().len() == 0")
    }
}

impl<T> RawSlide<T> {
    /// Peek the last `n` elements without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_back_slice_unchecked(&self, n: usize) -> NonNull<[T]> {
        unsafe {
            assert_unchecked!(
                n <= self.consumed().len(),
                "undefined behavior: `n > self.consumed().len()`"
            )
        };

        match self.borrow() {
            SlideRef::Zst(slide) => unsafe { slide.peek_back_slice_unchecked(n) },
            SlideRef::NonZst(slide) => unsafe { slide.peek_back_slice_unchecked(n) },
        }
    }

    /// Peek the last `N` elements as an array without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `N <= self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_back_array_unchecked<const N: usize>(&self) -> NonNull<[T; N]> {
        unsafe {
            assert_unchecked!(
                N <= self.consumed().len(),
                "undefined behavior: `N > self.consumed().len()`"
            )
        };

        unsafe { self.peek_back_slice_unchecked(N) }.cast()
    }

    /// Peek the last element without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `self.consumed().len() != 0`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_back_unchecked(&self) -> NonNull<T> {
        unsafe {
            assert_unchecked!(
                self.consumed().len() != 0,
                "undefined behavior: `self.consumed().len() == 0`"
            )
        };

        unsafe { self.peek_back_slice_unchecked(1) }.cast()
    }

    /// Peek the last `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_back_slice_checked(&self, n: usize) -> Option<NonNull<[T]>> {
        if n <= self.consumed().len() {
            Some(unsafe { self.peek_back_slice_unchecked(n) })
        } else {
            None
        }
    }

    /// Peek the last `N` elements as an array if `N <= self.consumed().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `N > self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_back_array_checked<const N: usize>(&self) -> Option<NonNull<[T; N]>> {
        match self.peek_back_slice_checked(N) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    /// Peek the last element if `self.consumed().len() != 0`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `self.consumed().len() == 0`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn peek_back_checked(&self) -> Option<NonNull<T>> {
        match self.peek_back_slice_checked(1) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }
}

impl<T> Clone for RawSlide<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RawSlide<T> {}

enum SlideRef<'a, T> {
    Zst(&'a zst::Slide<T>),
    NonZst(&'a non_zst::Slide<T>),
}

enum SlideMut<'a, T> {
    Zst(&'a mut zst::Slide<T>),
    NonZst(&'a mut non_zst::Slide<T>),
}
