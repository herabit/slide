use core::{
    fmt, hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::{Direction, Slide, raw::RawSlide};

/// A slide over a mutable buffer.
#[repr(transparent)]
pub struct SlideMut<'a, T> {
    raw: RawSlide<T>,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> SlideMut<'a, T> {
    #[inline]
    #[must_use]
    pub(crate) const unsafe fn from_raw(raw: RawSlide<T>) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// Create a new [`SlideMut`] starting at the start of the slice.
    #[inline]
    #[must_use]
    pub const fn new(slice: &'a mut [T]) -> Self {
        unsafe { Self::from_raw(RawSlide::from_slice_mut(slice)) }
    }

    /// Create a new [`SlideMut`] starting at `offset`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `offset` is out of bounds for `slice`.
    #[inline]
    #[must_use]
    pub const fn with_offset(slice: &'a mut [T], offset: usize) -> Option<Self> {
        match RawSlide::from_slice_mut_offset(slice, offset) {
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

    // #[inline]
    // #[must_use]
    // pub const fn as_slide(&self) -> Slide<'_, T> {
    //     unsafe { Slide::from_raw(self.raw) }
    // }

    // #[inline]
    // #[must_use]
    // pub const fn as_slide_mut(&mut self) -> SlideMut<'_, T> {
    //     unsafe { SlideMut::from_raw(self.raw) }
    // }
    //

    /// Convert this [`SlideMut`] into a [`Slide`].
    #[inline]
    #[must_use]
    pub const fn into_slide(self) -> Slide<'a, T> {
        unsafe { Slide::from_raw(self.raw) }
    }
}

impl<'a, T> SlideMut<'a, T> {
    /// Returns a reference to the source slice.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &[T] {
        unsafe { self.raw.source().as_ref() }
    }

    /// Returns a mutable reference to the source slice.
    #[inline]
    #[must_use]
    pub const fn source_mut(&mut self) -> &mut [T] {
        unsafe { self.raw.source().as_mut() }
    }

    /// Convert this [`SlideMut`] into the source slice.
    #[inline]
    #[must_use]
    pub const fn into_source(self) -> &'a mut [T] {
        unsafe { self.raw.source().as_mut() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    /// Returns a reference to the consumed slice.
    #[inline]
    #[must_use]
    pub const fn consumed(&self) -> &[T] {
        unsafe { self.raw.consumed().as_ref() }
    }

    /// Returns a mutable reference to the consumed slice.
    #[inline]
    #[must_use]
    pub const fn consumed_mut(&mut self) -> &mut [T] {
        unsafe { self.raw.consumed().as_mut() }
    }

    /// Converts this [`SlideMut`] into the consumed slice.
    #[inline]
    #[must_use]
    pub const fn into_consumed(self) -> &'a mut [T] {
        unsafe { self.raw.consumed().as_mut() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    /// Returns a reference to the remaining slice.
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> &[T] {
        unsafe { self.raw.remaining().as_ref() }
    }

    /// Returns a mutable reference to the remaining slice.
    #[inline]
    #[must_use]
    pub const fn remaining_mut(&mut self) -> &mut [T] {
        unsafe { self.raw.remaining().as_mut() }
    }

    /// Convert this [`SlideMut`] into the remaining slice.
    #[inline]
    #[must_use]
    pub const fn into_remaining(self) -> &'a mut [T] {
        unsafe { self.raw.remaining().as_mut() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    /// Split consumed and remaining slices.
    #[inline]
    #[must_use]
    pub const fn split(&self) -> (&[T], &[T]) {
        (self.consumed(), self.remaining())
    }

    /// Mutably split the consumed and remaining slices.
    #[inline]
    #[must_use]
    pub const fn split_mut(&mut self) -> (&mut [T], &mut [T]) {
        unsafe { (self.raw.consumed().as_mut(), self.raw.remaining().as_mut()) }
    }

    /// Convert this [`SlideMut`] into the consumed and remaining slices.
    #[inline]
    #[must_use]
    pub const fn into_split(self) -> (&'a mut [T], &'a mut [T]) {
        unsafe { (self.raw.consumed().as_mut(), self.raw.remaining().as_mut()) }
    }
}

impl<'a, T> SlideMut<'a, T> {
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

impl<'a, T> SlideMut<'a, T> {
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

    /// Slide the cursor over in a digen direction.
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

impl<'a, T> SlideMut<'a, T> {
    /// Peek `n` elements in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `n <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_slice_unchecked(&self, dir: Direction, n: usize) -> &[T] {
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
    pub const unsafe fn peek_array_unchecked<const N: usize>(&self, dir: Direction) -> &[T; N] {
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
    pub const unsafe fn peek_unchecked(&self, dir: Direction) -> &T {
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
    pub const fn peek_slice_checked(&self, dir: Direction, n: usize) -> Option<&[T]> {
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
    pub const fn peek_array_checked<const N: usize>(&self, dir: Direction) -> Option<&[T; N]> {
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
    pub const fn peek_checked(&self, dir: Direction) -> Option<&T> {
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
    pub const fn peek_slice(&self, dir: Direction, n: usize) -> &[T] {
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
    pub const fn peek_array<const N: usize>(&self, dir: Direction) -> &[T; N] {
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
    pub const fn peek(&self, dir: Direction) -> &T {
        unsafe { self.raw.peek(dir).as_ref() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    /// Peek `n` elements mutably in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `n <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_mut_slice_unchecked(&mut self, dir: Direction, n: usize) -> &mut [T] {
        unsafe { self.raw.peek_slice_unchecked(dir, n).as_mut() }
    }

    /// Peek `N` elements mutably as an array in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `N <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `N <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_mut_array_unchecked<const N: usize>(
        &mut self,
        dir: Direction,
    ) -> &mut [T; N] {
        unsafe { self.raw.peek_array_unchecked::<N>(dir).as_mut() }
    }

    /// Peek the first element mutably in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `!self.remaining().is_empty()`.
    /// - [`Direction::Left`]: The caller must ensure `!self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_mut_unchecked(&mut self, dir: Direction) -> &mut T {
        unsafe { self.raw.peek_unchecked(dir).as_mut() }
    }

    /// Peek `n` elements mutably in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `n > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_mut_slice_checked(&mut self, dir: Direction, n: usize) -> Option<&mut [T]> {
        match self.raw.peek_slice_checked(dir, n) {
            Some(mut ptr) => Some(unsafe { ptr.as_mut() }),
            None => None,
        }
    }

    /// Peek `N` elements mutably as an array in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `N > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `N > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_mut_array_checked<const N: usize>(
        &mut self,
        dir: Direction,
    ) -> Option<&mut [T; N]> {
        match self.raw.peek_array_checked(dir) {
            Some(mut ptr) => Some(unsafe { ptr.as_mut() }),
            None => None,
        }
    }

    /// Peek the first element mutably in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `self.remaining().is_empty()`.
    /// - [`Direction::Left`]: Returns `None` if `self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_mut_checked(&mut self, dir: Direction) -> Option<&mut T> {
        match self.raw.peek_checked(dir) {
            Some(mut ptr) => Some(unsafe { ptr.as_mut() }),
            None => None,
        }
    }

    /// Peek `n` elements mutably in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `n > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_mut_slice(&mut self, dir: Direction, n: usize) -> &mut [T] {
        unsafe { self.raw.peek_slice(dir, n).as_mut() }
    }

    /// Peek `N` elements mutably as an array in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `N > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `N > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_mut_array<const N: usize>(&mut self, dir: Direction) -> &mut [T; N] {
        unsafe { self.raw.peek_array::<N>(dir).as_mut() }
    }

    /// Peek the next element mutably in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `self.remaining().is_empty()`.
    /// - [`Direction::Left`]: Panics if `self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_mut(&mut self, dir: Direction) -> &mut T {
        unsafe { self.raw.peek(dir).as_mut() }
    }
}

unsafe impl<'a, T> Send for SlideMut<'a, T> where &'a mut [T]: Send {}
unsafe impl<'a, T> Sync for SlideMut<'a, T> where &'a mut [T]: Sync {}

impl<'a, T: fmt::Debug> fmt::Debug for SlideMut<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (consumed, remaining) = self.split();

        f.debug_struct("SlideMut")
            .field("consumed", &consumed)
            .field("remaining", &remaining)
            .finish()
    }
}

impl<'a, 'b, T, U> PartialEq<SlideMut<'b, U>> for SlideMut<'a, T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SlideMut<'b, U>) -> bool {
        self.consumed() == other.consumed() && self.remaining() == other.remaining()
    }
}

impl<'a, 'b, T, U> PartialEq<Slide<'b, U>> for SlideMut<'a, T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Slide<'b, U>) -> bool {
        self.consumed() == other.consumed() && self.remaining() == other.remaining()
    }
}

impl<'a, T> Eq for SlideMut<'a, T> where T: Eq {}

impl<'a, 'b, T> PartialOrd<SlideMut<'b, T>> for SlideMut<'a, T>
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

impl<'a, 'b, T> PartialOrd<Slide<'b, T>> for SlideMut<'a, T>
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

impl<'a, T> Ord for SlideMut<'a, T>
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

impl<'a, T> hash::Hash for SlideMut<'a, T>
where
    T: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.consumed().hash(state);
        self.remaining().hash(state);
    }
}

impl<'a, T> Default for SlideMut<'a, T> {
    #[inline]
    fn default() -> Self {
        SlideMut::new(Default::default())
    }
}

impl<'a, T> From<&'a mut [T]> for SlideMut<'a, T> {
    #[inline]
    fn from(value: &'a mut [T]) -> Self {
        SlideMut::new(value)
    }
}

impl<'a, T, I> Index<I> for SlideMut<'a, T>
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

impl<'a, T, I> IndexMut<I> for SlideMut<'a, T>
where
    I: SliceIndex<[T]>,
{
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.remaining_mut().index_mut(index)
    }
}
