use core::ops::{Bound, Index, IndexMut, Range, RangeBounds};

use crate::macros::assert_unchecked;

/// A `start..end` range where `start <= end` is always true.
///
/// It contains all values where `start <= x < end`. It is empty
/// if, and only if `start == end`. `start > end` is considered
/// an ***impossible state***.
///
/// # Safety
///
/// Creating an [`IndexRange`] where `start > end` is not only
/// incorrect, but undefined behavior.
///
/// This type informs the compiler that `start <= end` is ***always true***,
/// and that for it to be false is undefined behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexRange {
    start: usize,
    end: usize,
}

impl IndexRange {
    /// Create a new [`IndexRange`] without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `start <= end`, and failure to
    /// do so is undefined behavior.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn new_unchecked(start: usize, end: usize) -> IndexRange {
        // SAFETY: The caller ensures `start <= end`.
        unsafe { assert_unchecked!(start <= end, "`start > end`") };

        IndexRange { start, end }
    }

    /// Create a new [`IndexRange`] if `start <= end`.
    ///
    /// # Returns
    ///
    /// Returns `None` if `start > end`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn new(start: usize, end: usize) -> Option<IndexRange> {
        if start <= end {
            // SAFETY: We checked that `start <= end`.
            Some(unsafe { IndexRange::new_unchecked(start, end) })
        } else {
            None
        }
    }

    /// Hint to the compiler that an [`IndexRange`] can only be created
    /// if `start <= end`.
    #[inline(always)]
    #[track_caller]
    pub const fn compiler_hints(&self) {
        // SAFETY: We know that `start <= end`.
        unsafe { assert_unchecked!(self.start <= self.end, "`start > end`") }
    }

    /// Get the length of the range.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn len(&self) -> usize {
        self.compiler_hints();

        unsafe { self.end.unchecked_sub(self.start) }
    }

    /// Get the start of the range.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn start(&self) -> usize {
        self.compiler_hints();

        self.start
    }

    /// Get the end of the range.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn end(&self) -> usize {
        self.compiler_hints();

        self.end
    }

    /// Get a reference to the start of the range.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn start_ref(&self) -> &usize {
        self.compiler_hints();

        &self.start
    }

    /// Get a reference to the end of the range.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn end_ref(&self) -> &usize {
        self.compiler_hints();

        &self.end
    }

    /// Get a mutable reference to the start and end
    /// of the range.
    ///
    /// # Safety
    ///
    /// The caller must ensure that before the borrows end
    /// and the underlying [`IndexRange`] is used that
    /// `start <= end` is true.
    ///
    /// Failure to uphold the invariants of [`IndexRange`]
    /// is undefined behavior.
    ///
    /// Use at your own risk.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn as_mut(&mut self) -> Range<&mut usize> {
        self.compiler_hints();

        // SAFETY: The caller ensures that when the borrows end,
        //         that `start <= end` is true.
        Range {
            start: &mut self.start,
            end: &mut self.end,
        }
    }

    /// Get a mutable reference to the start of the range.
    ///
    /// # Safety
    ///
    /// The caller must ensure that before the borrow ends
    /// and the underlying [`IndexRange`] is used that
    /// `start <= end` is true.
    ///
    /// Failure to uphold the invariants of [`IndexRange`]
    /// is undefined behavior.
    ///
    /// Use at your own risk.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn start_mut(&mut self) -> &mut usize {
        // SAFETY: The caller ensures that when the borrow
        //         ends, that `start <= end` is true.
        unsafe { self.as_mut().start }
    }

    /// Get a mutable reference to the end of the range.
    ///
    /// # Safety
    ///
    /// The caller must ensure that before the borrow ends
    /// and the underlying [`IndexRange`] is used that
    /// `start <= end` is true.
    ///
    /// Failure to uphold the invariants of [`IndexRange`]
    /// is undefined behavior.
    ///
    /// Use at your own risk.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn end_mut(&mut self) -> &mut usize {
        // SAFETY: The caller ensures that when the borrow
        //         ends, that `start <= end` is true.
        unsafe { self.as_mut().end }
    }
}

impl Default for IndexRange {
    #[inline(always)]
    fn default() -> Self {
        IndexRange::new(0, 0).unwrap()
    }
}

impl RangeBounds<usize> for IndexRange {
    #[inline(always)]
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(self.start_ref())
    }

    #[inline(always)]
    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(self.end_ref())
    }
}

impl IntoIterator for IndexRange {
    type Item = usize;
    type IntoIter = <Range<usize> as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        (self.start()..self.end()).into_iter()
    }
}

impl From<IndexRange> for Range<usize> {
    #[inline(always)]
    fn from(value: IndexRange) -> Self {
        value.start()..value.end()
    }
}

impl<T> Index<IndexRange> for [T] {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: IndexRange) -> &Self::Output {
        &self[Range::from(index)]
    }
}

impl<T> IndexMut<IndexRange> for [T] {
    #[inline(always)]
    fn index_mut(&mut self, index: IndexRange) -> &mut Self::Output {
        &mut self[Range::from(index)]
    }
}

impl Index<IndexRange> for str {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: IndexRange) -> &Self::Output {
        &self[Range::from(index)]
    }
}

impl IndexMut<IndexRange> for str {
    #[inline(always)]
    fn index_mut(&mut self, index: IndexRange) -> &mut Self::Output {
        &mut self[Range::from(index)]
    }
}

#[unsafe(no_mangle)]
fn lol(s: &[u8], r: IndexRange) -> &[u8] {
    &s[r]
}
