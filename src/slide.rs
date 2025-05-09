use core::{fmt, hash, marker::PhantomData, ops::Index, slice::SliceIndex};

use crate::{raw::RawSlide, util::nonnull_from_ref};

#[repr(transparent)]
pub struct Slide<'a, T> {
    raw: RawSlide<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Slide<'a, T> {
    /// Create a new [`Slide`] from a `slice` and `offset` without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `offset <= slice.len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_parts_unchecked(slice: &'a [T], offset: usize) -> Self {
        Self {
            raw: unsafe { RawSlide::from_parts_unchecked(nonnull_from_ref(slice), offset) },
            _marker: PhantomData,
        }
    }

    /// Create a new [`Slide`] from a `slice` and `offset` if `offset <= slice.len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `offset > slice.len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_parts_checked(slice: &'a [T], offset: usize) -> Option<Self> {
        if offset <= slice.len() {
            Some(unsafe { Self::from_parts_unchecked(slice, offset) })
        } else {
            None
        }
    }

    /// Create a new [`Slide`] from a `slice` and `offset`.
    ///
    /// # Panics
    ///
    /// Panics if `offset > slice.len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_parts(slice: &'a [T], offset: usize) -> Self {
        Self::from_parts_checked(slice, offset).expect("offset > slice.len()")
    }

    /// Create a new [`Slide`] from a `slice`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn new(slice: &'a [T]) -> Self {
        Self::from_parts(slice, 0)
    }

    /// Provide hints to the compiler.
    #[inline(always)]
    #[track_caller]
    pub const fn compiler_hints(&self) {
        self.raw.compiler_hints()
    }
}

impl<'a, T> Slide<'a, T> {
    /// Returns whether the slide is fully consumed.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_consumed(&self) -> bool {
        self.raw.is_consumed()
    }

    /// Return the source slice.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn source(&self) -> &'a [T] {
        unsafe { self.raw.source().as_ref() }
    }

    /// Return the consumed slice.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn consumed(&self) -> &'a [T] {
        unsafe { self.raw.consumed().as_ref() }
    }

    /// Return the remaining slice.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn remaining(&self) -> &'a [T] {
        unsafe { self.raw.remaining().as_ref() }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn split(&self) -> (&'a [T], &'a [T]) {
        (self.consumed(), self.remaining())
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn parts(&self) -> (&'a [T], usize) {
        (self.source(), self.offset())
    }
}

impl<'a, T> Slide<'a, T> {
    /// Returns the offset for the cursor.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn offset(&self) -> usize {
        self.raw.offset()
    }

    /// Updates the current offset of the cursor without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `offset <= self.source().len()`.
    #[inline(always)]
    #[track_caller]
    pub const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe { self.raw.set_offset_unchecked(offset) }
    }

    /// Updates the current offset of the cursor if `offset <= self.source().len()`.
    ///
    /// # Returns
    ///
    ///  Returns whether the cursor's offset was updated.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn set_offset_checked(&mut self, offset: usize) -> bool {
        self.raw.set_offset_checked(offset)
    }

    /// Updates the current offset of the cursor.
    ///
    /// # Panics
    ///
    /// Panics if `offset > self.source().len()`.
    #[inline(always)]
    #[track_caller]
    pub const fn set_offset(&mut self, offset: usize) {
        self.raw.set_offset(offset)
    }
}

impl<'a, T> Slide<'a, T> {
    /// Peek the next `n` elements without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_unchecked(&self, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_unchecked(n).as_ref() }
    }

    /// Peek the previous `n` elements without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_back_unchecked(&self, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_back_unchecked(n).as_ref() }
    }

    /// Advance the next `n` elements without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.remaining().len()`.
    #[inline(always)]
    #[track_caller]
    pub const unsafe fn advance_unchecked(&mut self, n: usize) -> &'a [T] {
        unsafe { self.raw.advance_unchecked(n).as_ref() }
    }

    /// Rewind the previous `n` elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure `n <= self.consumed().len()`.
    #[inline(always)]
    #[track_caller]
    pub const unsafe fn rewind_unchecked(&mut self, n: usize) -> &'a [T] {
        unsafe { self.raw.rewind_unchecked(n).as_ref() }
    }
}

impl<'a, T> Slide<'a, T> {
    /// Peek the next `n` elements if `n <= self.remaining().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn peek_checked(&self, n: usize) -> Option<&'a [T]> {
        match self.raw.peek_checked(n) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    /// Peek the previous `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn peek_back_checked(&self, n: usize) -> Option<&'a [T]> {
        match self.raw.peek_back_checked(n) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    /// Advance the next `n` elements if `n <= self.remaining().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.remaining().len()`.
    #[inline(always)]
    #[track_caller]
    pub const fn advance_checked(&mut self, n: usize) -> Option<&'a [T]> {
        match self.raw.advance_checked(n) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    /// Rewind the previous `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `n > self.consumed().len()`.
    #[inline(always)]
    #[track_caller]
    pub const fn rewind_checked(&mut self, n: usize) -> Option<&'a [T]> {
        match self.raw.rewind_checked(n) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }
}

impl<'a, T> Slide<'a, T> {
    /// Peek the next `n` elements if `n <= self.remaining().len()`.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.remaining().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn peek(&self, n: usize) -> &'a [T] {
        unsafe { self.raw.peek(n).as_ref() }
    }

    /// Peek the previous `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.consumed().len()`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn peek_back(&self, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_back(n).as_ref() }
    }

    /// Advance the next `n` elements if `n <= self.remaining().len()`.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.remaining().len()`.
    #[inline(always)]
    #[track_caller]
    pub const fn advance(&mut self, n: usize) -> &'a [T] {
        unsafe { self.raw.advance(n).as_ref() }
    }

    /// Rewind the previous `n` elements if `n <= self.consumed().len()`.
    ///
    /// # Panics
    ///
    /// Panics if `n > self.consumed().len()`.
    #[inline(always)]
    #[track_caller]
    pub const fn rewind(&mut self, n: usize) -> &'a [T] {
        unsafe { self.raw.rewind(n).as_ref() }
    }
}

impl<'a, T> Clone for Slide<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Slide<'a, T> {}

unsafe impl<'a, T> Send for Slide<'a, T> where &'a [T]: Send {}
unsafe impl<'a, T> Sync for Slide<'a, T> where &'a [T]: Sync {}

impl<'a, T: PartialEq> PartialEq for Slide<'a, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.parts() == other.parts()
    }

    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        self.parts() != other.parts()
    }
}

impl<'a, T: Eq> Eq for Slide<'a, T> {}

impl<'a, T: PartialOrd> PartialOrd for Slide<'a, T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.parts().partial_cmp(&other.parts())
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        self.parts() < other.parts()
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        self.parts() <= other.parts()
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        self.parts() > other.parts()
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        self.parts() >= other.parts()
    }
}

impl<'a, T: Ord> Ord for Slide<'a, T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.parts().cmp(&other.parts())
    }
}

impl<'a, T: hash::Hash> hash::Hash for Slide<'a, T> {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.parts().hash(state)
    }
}

impl<'a, T> Default for Slide<'a, T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(&[])
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Slide<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (consumed, remaining) = self.parts();

        f.debug_struct("Slide")
            .field("consumed", &consumed)
            .field("remaining", &remaining)
            .finish()
    }
}

impl<'a, T> From<&'a [T]> for Slide<'a, T> {
    #[inline(always)]
    fn from(value: &'a [T]) -> Self {
        Self::new(value)
    }
}

impl<'a, T> AsRef<[T]> for Slide<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self.remaining()
    }
}

impl<'a, T, I> Index<I> for Slide<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline(always)]
    #[track_caller]
    fn index(&self, index: I) -> &Self::Output {
        self.remaining().index(index)
    }
}

#[unsafe(no_mangle)]
fn lol(x: &mut Slide<u8>) {
    while x.remaining().len() >= 5 {
        x.advance(5);
    }
}

#[unsafe(no_mangle)]
unsafe fn lol_2((start, end): &mut (*const u8, *const u8)) {
    while unsafe { end.offset_from(*start) >= 5 } {
        *start = unsafe { start.add(5) };
    }
}
