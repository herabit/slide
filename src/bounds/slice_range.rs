use core::{
    error, fmt,
    num::NonZero,
    ops::{self, Bound, Index, IndexMut, Range, RangeBounds},
};

use crate::macros::{assert_unchecked, unreachable_unchecked};

use super::{SliceBounds, as_bounds, into_bounds, to_bounds};

/// A `start..end` range where `start <= end` is always true.
///
/// It contains all values where `start <= x < end`. It is empty
/// if, and only if `start == end`. `start > end` is considered
/// an ***impossible state***.
///
/// # Safety
///
/// Creating a [`SliceRange`] where `start > end` is not only
/// incorrect, but undefined behavior.
///
/// This type informs the compiler that `start <= end` is ***always true***,
/// and that for it to be false is undefined behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliceRange {
    start: usize,
    end: usize,
}

impl SliceRange {
    /// Attempt to create a new [`SliceRange`] from `start..end`.
    ///
    /// # Returns
    ///
    /// Returns an error if `start > end.
    #[inline(always)]
    #[must_use]
    pub const fn try_new(start: usize, end: usize) -> Result<SliceRange, SliceRangeError> {
        if start > end {
            Err(SliceRangeError::StartTooLarge {
                start: NonZero::new(start).unwrap(),
                end,
            })
        } else {
            // SAFETY: We checked that `start <= end`.
            Ok(SliceRange { start, end })
        }
    }

    /// Create a new [`SliceRange`] from `start..end`.
    ///
    /// # Panics
    ///
    /// Panics if `start > end`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn new(start: usize, end: usize) -> SliceRange {
        match SliceRange::try_new(start, end) {
            Ok(range) => range,
            Err(err) => err.handle(),
        }
    }

    /// Create a new [`SliceRange`] from `start..end` without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `start <= end`. Creating a [`SliceRange`]
    /// where `start > end` is undefined behavior.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn new_unchecked(start: usize, end: usize) -> SliceRange {
        match SliceRange::try_new(start, end) {
            Ok(range) => range,
            // SAFETY: The caller ensures `start <= end`.
            Err(err) => unsafe { err.handle_unreachable() },
        }
    }

    /// Create a new [`SliceRange`] from `0..end`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn up_to(end: usize) -> SliceRange {
        SliceRange::new(0, end)
    }

    /// Hint to the compiler that an [`SliceRange`] can only be created
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
    /// and the underlying [`SliceRange`] is used that
    /// `start <= end` is true.
    ///
    /// Failure to uphold the invariants of [`SliceRange`]
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
    /// and the underlying [`SliceRange`] is used that
    /// `start <= end` is true.
    ///
    /// Failure to uphold the invariants of [`SliceRange`]
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
    /// and the underlying [`SliceRange`] is used that
    /// `start <= end` is true.
    ///
    /// Failure to uphold the invariants of [`SliceRange`]
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

impl SliceRange {
    /// Attempt to create a new [`SliceRange`] from something that
    /// implements [`SliceBounds`].
    ///
    /// # Returns
    ///
    /// This method returns an error if:
    ///
    /// - The start index would overflow.
    /// - The end index would overflow.
    /// - The start index is greater than the end index (`start > end`).
    /// - The end index is greater than the length (`end > len`).
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn try_from_slice_bounds<B: SliceBounds + ?Sized>(
        bounds: &B,
        len: usize,
    ) -> Result<SliceRange, SliceRangeError> {
        try_from_bounds(as_bounds(bounds), len)
    }

    /// Attempt to create a new [`SliceRange`] from something that
    /// implements [`RangeBounds`].
    ///
    /// # Returns
    ///
    /// This method returns an error if:
    ///
    /// - The start index would overflow.
    /// - The end index would overflow.
    /// - The start index is greater than the end index (`start > end`).
    /// - The end index is greater than the length (`end > len`).
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub fn try_from_range_bounds<B: RangeBounds<usize> + ?Sized>(
        bounds: &B,
        len: usize,
    ) -> Result<SliceRange, SliceRangeError> {
        try_from_bounds((bounds.start_bound(), bounds.end_bound()), len)
    }

    /// Create a new [`SliceRange`] from something that implements [`SliceBounds`].
    ///
    /// # Panics
    ///
    /// This method panics if:
    ///
    /// - The start index would overflow.
    /// - The end index would overflow.
    /// - The start index is greater than the end index (`start > end`).
    /// - The end index is greater than the length (`end > len`).
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_slice_bounds<B: SliceBounds + ?Sized>(bounds: &B, len: usize) -> SliceRange {
        match try_from_bounds(as_bounds(bounds), len) {
            Ok(range) => range,
            Err(err) => err.handle(),
        }
    }

    /// Create a new [`SliceRange`] from something that implements [`RangeBounds`].
    ///
    /// # Panics
    ///
    /// This method panics if:
    ///
    /// - The start index would overflow.
    /// - The end index would overflow.
    /// - The start index is greater than the end index (`start > end`).
    /// - The end index is greater than the length (`end > len`).
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub fn from_range_bounds<B: RangeBounds<usize> + ?Sized>(bounds: &B, len: usize) -> SliceRange {
        match try_from_bounds((bounds.start_bound(), bounds.end_bound()), len) {
            Ok(range) => range,
            Err(err) => panic!("{err}"),
        }
    }

    /// Create a new [`SliceRange`] from something that implements [`SliceBounds`] without any checks.
    ///
    /// # Safety
    ///
    /// This method results in undefined behavior if:
    ///
    /// - The start index overflows.
    /// - The end index overflows.
    /// - The start index is greater than the end index (`start > end`).
    /// - The end index is greater than the length (`end > len`).
    ///
    /// The caller is responsible for ensuring the above cases are impossible.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_slice_bounds_unchecked<B: SliceBounds + ?Sized>(
        bounds: &B,
        len: usize,
    ) -> SliceRange {
        match try_from_bounds(as_bounds(bounds), len) {
            Ok(range) => range,
            // SAFETY: The caller ensures that this can never occur.
            Err(err) => unsafe { err.handle_unreachable() },
        }
    }

    /// Create a new [`SliceRange`] from something that implements [`RangeBounds`] without any checks.
    ///
    /// # Safety
    ///
    /// The method results in undefined behavior if:
    ///
    /// - The start index overflows.
    /// - The end index overflows.
    /// - The start index is greater than the end index (`start > end`).
    /// - The end index is greater than the length (`end > len`).
    ///
    /// The caller is responsible for ensuring the above cases are impossible.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub unsafe fn from_range_bounds_unchecked<B: RangeBounds<usize> + ?Sized>(
        bounds: &B,
        len: usize,
    ) -> SliceRange {
        match try_from_bounds((bounds.start_bound(), bounds.end_bound()), len) {
            Ok(range) => range,
            // SAFETY: The caller ensures this can never occur.
            Err(err) => unsafe { unreachable_unchecked!("{}", err) },
        }
    }
}

/// Attempt to create a new [`SliceRange`] given the start and end bounds
/// of some bounded type.
///
/// This is mainly an implementation detail of the publicly exposed methods on [`SliceRange`].
///
/// # Returns
///
/// This method returns an error if:
///
/// - The start index would overflow.
/// - The end index would overflow.
/// - The start index is greater than the end index (`start > end`).
/// - The end index is greater than the length (`end > len`).
#[inline(always)]
#[must_use]
const fn try_from_bounds(
    (start, end): (Bound<&usize>, Bound<&usize>),
    len: usize,
) -> Result<SliceRange, SliceRangeError> {
    let start = match start {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => match start.checked_add(1) {
            Some(start) => start,
            None => return Err(SliceRangeError::StartOverflow),
        },
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(&end) => match end.checked_add(1) {
            Some(end) => end,
            None => return Err(SliceRangeError::EndOverflow),
        },
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    if start > end {
        Err(SliceRangeError::StartTooLarge {
            start: NonZero::new(start).unwrap(),
            end,
        })
    } else if end > len {
        Err(SliceRangeError::EndTooLarge {
            end: NonZero::new(end).unwrap(),
            len,
        })
    } else {
        // SAFETY: We know that `start <= end && end <= len`.
        Ok(SliceRange { start, end })
    }
}

impl Default for SliceRange {
    #[inline(always)]
    fn default() -> Self {
        SliceRange::up_to(0)
    }
}

impl RangeBounds<usize> for SliceRange {
    #[inline(always)]
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(self.start_ref())
    }

    #[inline(always)]
    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(self.end_ref())
    }
}

impl From<&SliceRange> for SliceRange {
    #[inline(always)]
    fn from(value: &SliceRange) -> Self {
        *value
    }
}

impl From<SliceRange> for ops::Range<usize> {
    #[inline(always)]
    fn from(value: SliceRange) -> Self {
        value.start()..value.end()
    }
}

impl<'a> From<&'a SliceRange> for ops::Range<&'a usize> {
    #[inline(always)]
    fn from(value: &'a SliceRange) -> Self {
        value.start_ref()..value.end_ref()
    }
}

impl From<&SliceRange> for ops::Range<usize> {
    #[inline(always)]
    fn from(value: &SliceRange) -> Self {
        value.start()..value.end()
    }
}

impl From<SliceRange> for (Bound<usize>, Bound<usize>) {
    #[inline(always)]
    fn from(value: SliceRange) -> Self {
        into_bounds(value)
    }
}

impl<'a> From<&'a SliceRange> for (Bound<&'a usize>, Bound<&'a usize>) {
    #[inline(always)]
    fn from(value: &'a SliceRange) -> Self {
        as_bounds(value)
    }
}

impl From<&SliceRange> for (Bound<usize>, Bound<usize>) {
    #[inline(always)]
    fn from(value: &SliceRange) -> Self {
        to_bounds(value)
    }
}

impl<T> TryFrom<ops::Range<T>> for SliceRange
where
    ops::Range<T>: RangeBounds<usize>,
{
    type Error = SliceRangeError;

    #[inline(always)]
    fn try_from(value: ops::Range<T>) -> Result<Self, Self::Error> {
        SliceRange::try_from_range_bounds(&value, usize::MAX)
    }
}

impl<T> TryFrom<&ops::Range<T>> for SliceRange
where
    ops::Range<T>: RangeBounds<usize>,
{
    type Error = SliceRangeError;

    #[inline(always)]
    fn try_from(value: &ops::Range<T>) -> Result<Self, Self::Error> {
        SliceRange::try_from_range_bounds(value, usize::MAX)
    }
}

impl<T> TryFrom<ops::RangeInclusive<T>> for SliceRange
where
    ops::RangeInclusive<T>: RangeBounds<usize>,
{
    type Error = SliceRangeError;

    #[inline(always)]
    fn try_from(value: ops::RangeInclusive<T>) -> Result<Self, Self::Error> {
        SliceRange::try_from_range_bounds(&value, usize::MAX)
    }
}

impl<T> TryFrom<&ops::RangeInclusive<T>> for SliceRange
where
    ops::RangeInclusive<T>: RangeBounds<usize>,
{
    type Error = SliceRangeError;

    #[inline(always)]
    fn try_from(value: &ops::RangeInclusive<T>) -> Result<Self, Self::Error> {
        SliceRange::try_from_range_bounds(value, usize::MAX)
    }
}

impl<T> Index<SliceRange> for [T] {
    type Output = [T];

    #[inline(always)]
    #[track_caller]
    fn index(&self, index: SliceRange) -> &Self::Output {
        &self[ops::Range::from(index)]
    }
}

impl<T> IndexMut<SliceRange> for [T] {
    #[inline(always)]
    fn index_mut(&mut self, index: SliceRange) -> &mut Self::Output {
        &mut self[ops::Range::from(index)]
    }
}

impl<T> Index<&SliceRange> for [T] {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: &SliceRange) -> &Self::Output {
        &self[*index]
    }
}

impl<T> IndexMut<&SliceRange> for [T] {
    #[inline(always)]
    fn index_mut(&mut self, index: &SliceRange) -> &mut Self::Output {
        &mut self[*index]
    }
}

impl Index<SliceRange> for str {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: SliceRange) -> &Self::Output {
        &self[ops::Range::from(index)]
    }
}

impl IndexMut<SliceRange> for str {
    #[inline(always)]
    fn index_mut(&mut self, index: SliceRange) -> &mut Self::Output {
        &mut self[ops::Range::from(index)]
    }
}

impl Index<&SliceRange> for str {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: &SliceRange) -> &Self::Output {
        &self[*index]
    }
}

impl IndexMut<&SliceRange> for str {
    #[inline(always)]
    fn index_mut(&mut self, index: &SliceRange) -> &mut Self::Output {
        &mut self[*index]
    }
}

/// An error that occurs when creating an [`SliceRange`] bounds fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SliceRangeError {
    /// The range's start index would be greater than [`usize::MAX`],
    /// and therefore overflow.
    StartOverflow,
    /// The range's end index would be greater than [`usize::MAX`],
    /// and therefore overflow.
    EndOverflow,
    /// The range's start index is greater than its end index.
    StartTooLarge {
        /// The problematic start index.
        start: NonZero<usize>,
        /// The end index that the start index is greater than.
        end: usize,
    },
    /// The range's end index is greater than the slice length.
    EndTooLarge {
        /// The problematic end index.
        end: NonZero<usize>,
        /// The specified slice length the end index is greater than.
        len: usize,
    },
}

impl SliceRangeError {
    /// Panic with an error message if this error was hit.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    #[cold]
    pub(crate) const fn handle(&self) -> ! {
        match self {
            SliceRangeError::StartOverflow => panic!("range start would overflow"),
            SliceRangeError::EndOverflow => panic!("range end would overflow"),
            SliceRangeError::StartTooLarge { .. } => {
                panic!("range start is greater than end")
            }
            SliceRangeError::EndTooLarge { .. } => {
                panic!("range end is greater than slice length")
            }
        }
    }

    /// Assert that the code path producing this error is impossible.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this error is ***actually impossible to reach***.
    ///
    /// Failure to do so is undefined behavior.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    #[cold]
    pub(crate) const unsafe fn handle_unreachable(self) -> ! {
        // SAFETY: The caller ensures this is fine.
        unsafe {
            match self {
                SliceRangeError::StartOverflow => {
                    unreachable_unchecked!("range start would overflow")
                }
                SliceRangeError::EndOverflow => unreachable_unchecked!("range end would overflow"),
                SliceRangeError::StartTooLarge { .. } => {
                    unreachable_unchecked!("range start is greater than end")
                }
                SliceRangeError::EndTooLarge { .. } => {
                    unreachable_unchecked!("range end is greater than slice length")
                }
            }
        }
    }
}

impl fmt::Display for SliceRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SliceRangeError::StartOverflow => core::write!(f, "range start would overflow"),
            SliceRangeError::EndOverflow => core::write!(f, "range end would overflow"),
            SliceRangeError::StartTooLarge { start, end } => {
                core::write!(f, "range start {start} is greater than end {end}")
            }
            SliceRangeError::EndTooLarge { end, len } => {
                core::write!(f, "range end {end} is greater than slice length {len}")
            }
        }
    }
}

impl error::Error for SliceRangeError {}
