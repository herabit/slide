use core::{fmt, hash, marker::PhantomData, ops::Index, slice::SliceIndex};

use crate::{Direction, SlideMut, raw::RawSlide};

/// A slide over an immutable buffer.
#[repr(transparent)]
pub struct Slide<'a, T> {
    raw: RawSlide<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Slide<'a, T> {
    #[inline]
    #[must_use]
    pub(crate) const unsafe fn from_raw(raw: RawSlide<T>) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// Create a new [`Slide`] starting at the start of the slice.
    #[inline]
    #[must_use]
    pub const fn new(slice: &'a [T]) -> Self {
        unsafe { Self::from_raw(RawSlide::from_slice(slice)) }
    }

    /// Create a new [`Slide`] starting at `offset`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `offset` is out of bounds for `slice`.
    #[inline]
    #[must_use]
    pub const fn with_offset(slice: &'a [T], offset: usize) -> Option<Self> {
        match RawSlide::from_slice_offset(slice, offset) {
            Some(raw) => Some(unsafe { Self::from_raw(raw) }),
            None => None,
        }
    }

    /// Returns the offset of the cursor within the buffer.
    #[inline]
    #[must_use]
    pub const fn offset(&self) -> usize {
        let (offset, _) = self.raw.offset_len();

        offset
    }

    // /// This creates a new slide with a smaller lifetime.
    // #[inline]
    // #[must_use]
    // pub const fn as_slide(&self) -> Slide<'_, T> {
    //     *self
    // }

    /// Returns a reference to the source slice.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &'a [T] {
        unsafe { self.raw.source().as_ref() }
    }

    /// Returns a reference to the consumed slice.
    #[inline]
    #[must_use]
    pub const fn consumed(&self) -> &'a [T] {
        unsafe { self.raw.consumed().as_ref() }
    }

    /// Returns a reference to the remaining slice.
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> &'a [T] {
        unsafe { self.raw.remaining().as_ref() }
    }

    /// Split the source buffer at the cursor.
    #[inline]
    #[must_use]
    pub const fn split(&self) -> (&'a [T], &'a [T]) {
        (self.consumed(), self.remaining())
    }
}

impl<'a, T> Slide<'a, T> {
    /// Set the offset for the cursor without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `offset <= self.source().len()`.
    #[inline]
    #[track_caller]
    pub const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe { self.raw.set_offset_unchecked(offset) }
    }

    /// Set the offset for the cursor.
    ///
    /// # Returns
    ///
    /// Returns `offset <= self.source().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn set_offset_checked(&mut self, offset: usize) -> bool {
        self.raw.set_offset_checked(offset).is_some()
    }

    /// Set the offset for the cursor.
    ///
    /// # Panics
    ///
    /// Panics if `offset > self.source().len()`.
    #[inline]
    #[track_caller]
    pub const fn set_offset(&mut self, offset: usize) {
        self.raw.set_offset(offset)
    }
}

impl<'a, T> Slide<'a, T> {
    /// Slide the cursor over in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure that `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure that `n <= self.consumed().len()`.
    #[inline]
    #[track_caller]
    pub const unsafe fn slide_unchecked(&mut self, dir: Direction, n: usize) {
        unsafe { self.raw.slide_unchecked(dir, n) }
    }

    /// Slide the cursor over in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `n <= self.consumed().len()`.
    #[inline]
    #[track_caller]
    pub const fn slide_checked(&mut self, dir: Direction, n: usize) -> bool {
        self.raw.slide_checked(dir, n).is_some()
    }

    /// Slide the cursor over in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `n > self.consumed().len()`.
    #[inline]
    #[track_caller]
    pub const fn slide(&mut self, dir: Direction, n: usize) {
        self.raw.slide(dir, n)
    }
}

impl<'a, T> Slide<'a, T> {
    /// Peek `n` elements in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `n <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_slice_unchecked(&self, dir: Direction, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_slice_unchecked(dir, n).as_ref() }
    }

    /// Peek `N` elements as an array in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `N <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `N <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_array_unchecked<const N: usize>(&self, dir: Direction) -> &'a [T; N] {
        unsafe { self.raw.peek_array_unchecked::<N>(dir).as_ref() }
    }

    /// Peek the first element in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `!self.remaining().is_empty()`.
    /// - [`Direction::Left`]: The caller must ensure `!self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_unchecked(&self, dir: Direction) -> &'a T {
        unsafe { self.raw.peek_unchecked(dir).as_ref() }
    }

    /// Peek `n` elements in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `n > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_slice_checked(&self, dir: Direction, n: usize) -> Option<&'a [T]> {
        match self.raw.peek_slice_checked(dir, n) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    /// Peek `N` elements as an array in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `N > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `N > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_array_checked<const N: usize>(&self, dir: Direction) -> Option<&'a [T; N]> {
        match self.raw.peek_array_checked(dir) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    /// Peek the first element in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `self.remaining().is_empty()`.
    /// - [`Direction::Left`]: Returns `None` if `self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_checked(&self, dir: Direction) -> Option<&'a T> {
        match self.raw.peek_checked(dir) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    /// Peek `n` elements in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `n > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_slice(&self, dir: Direction, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_slice(dir, n).as_ref() }
    }

    /// Peek `N` elements as an array in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `N > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `N > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_array<const N: usize>(&self, dir: Direction) -> &'a [T; N] {
        unsafe { self.raw.peek_array::<N>(dir).as_ref() }
    }

    /// Peek the next element in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `self.remaining().is_empty()`.
    /// - [`Direction::Left`]: Panics if `self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek(&self, dir: Direction) -> &'a T {
        unsafe { self.raw.peek(dir).as_ref() }
    }
}

impl<'a, T> Clone for Slide<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Slide<'a, T> {}

unsafe impl<'a, T> Send for Slide<'a, T> where &'a [T]: Send {}
unsafe impl<'a, T> Sync for Slide<'a, T> where &'a [T]: Sync {}

impl<'a, T: fmt::Debug> fmt::Debug for Slide<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (consumed, remaining) = self.split();

        f.debug_struct("Slide")
            .field("consumed", &consumed)
            .field("remaining", &remaining)
            .finish()
    }
}

impl<'a, 'b, T, U> PartialEq<Slide<'b, U>> for Slide<'a, T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Slide<'b, U>) -> bool {
        self.consumed() == other.consumed() && self.remaining() == other.remaining()
    }
}

impl<'a, 'b, T, U> PartialEq<SlideMut<'b, U>> for Slide<'a, T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SlideMut<'b, U>) -> bool {
        self.consumed() == other.consumed() && self.remaining() == other.remaining()
    }
}

impl<'a, T> Eq for Slide<'a, T> where T: Eq {}

impl<'a, 'b, T> PartialOrd<Slide<'b, T>> for Slide<'a, T>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Slide<'b, T>) -> Option<core::cmp::Ordering> {
        let consumed = self.consumed().partial_cmp(other.consumed())?;
        let remaining = self.remaining().partial_cmp(other.remaining())?;

        Some(consumed.then(remaining))
    }
}

impl<'a, 'b, T> PartialOrd<SlideMut<'b, T>> for Slide<'a, T>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &SlideMut<'b, T>) -> Option<core::cmp::Ordering> {
        let consumed = self.consumed().partial_cmp(other.consumed())?;
        let remaining = self.remaining().partial_cmp(other.remaining())?;

        Some(consumed.then(remaining))
    }
}

impl<'a, T> Ord for Slide<'a, T>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.consumed()
            .cmp(other.consumed())
            .then_with(|| self.remaining().cmp(other.remaining()))
    }
}

impl<'a, T> hash::Hash for Slide<'a, T>
where
    T: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.consumed().hash(state);
        self.remaining().hash(state);
    }
}

impl<'a, T> Default for Slide<'a, T> {
    #[inline]
    fn default() -> Self {
        Slide::new(Default::default())
    }
}

impl<'a, T> From<&'a [T]> for Slide<'a, T> {
    #[inline]
    fn from(value: &'a [T]) -> Self {
        Slide::new(value)
    }
}

impl<'a, T> From<&'a mut [T]> for Slide<'a, T> {
    #[inline]
    fn from(value: &'a mut [T]) -> Self {
        Slide::new(value)
    }
}

impl<'a, T, I> Index<I> for Slide<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    #[inline]
    #[track_caller]
    fn index(&self, index: I) -> &Self::Output {
        self.remaining().index(index)
    }
}
