use core::{
    cmp::Ordering,
    error::Error,
    fmt, hash, mem,
    num::{NonZero, TryFromIntError},
};

use crate::{
    macros::{assert_unchecked, unreachable_unchecked},
    slice::Slice,
};

/// The error type that is returned when creating
/// a slice from its component elements.
///
/// ***TODO***
#[repr(transparent)]
pub struct FromElemsError<S>(pub S::FromElemsErr)
where
    S: Slice + ?Sized;

impl<S> FromElemsError<S>
where
    S: Slice + ?Sized,
{
    /// Panics with an error message corresponding to this error.
    #[track_caller]
    #[cold]
    #[inline(never)]
    pub const fn panic(self) -> ! {
        S::KIND.0.handle_from_elems_error(self.0)
    }

    /// Marks the code path that produced this error as impossible.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is impossible for this error
    /// to have been produced. Failure to do so is *undefined behavior* as
    /// calling this method makes promises to the compiler about what
    /// optimizations are valid.
    ///
    /// Proceed with caution.
    #[track_caller]
    #[cold]
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn panic_unchecked(self) -> ! {
        // SAFETY: The caller ensures it is impossible to reach this code.
        unsafe { S::KIND.0.handle_from_elems_error_unchecked(self.0) }
    }
}

impl<S> fmt::Debug for FromElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> fmt::Display for FromElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> Default for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S> Clone for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(
        &mut self,
        source: &Self,
    ) {
        self.0.clone_from(&source.0);
    }
}

impl<S> Copy for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Copy,
{
}

impl<S> PartialEq for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: PartialEq,
{
    #[inline]
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0 == other.0
    }
}

impl<S> Eq for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Eq,
{
}

impl<S> PartialOrd for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: PartialOrd,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Ord,
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> hash::Hash for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.0.hash(state);
    }
}

impl<S> core::error::Error for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: core::error::Error,
{
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0.description()
    }
}

/// The error type that is returned when trying to get a slice's
/// component elements.
///
/// ***TODO***
#[repr(transparent)]
pub struct AsElemsError<S>(pub S::AsElemsErr)
where
    S: Slice + ?Sized;

impl<S> AsElemsError<S>
where
    S: Slice + ?Sized,
{
    /// Panics with an error message corresponding to this error.
    #[track_caller]
    #[cold]
    #[inline(never)]
    pub const fn panic(self) -> ! {
        S::KIND.0.handle_as_elems_error(self.0)
    }

    /// Marks the code path that produced this error as impossible.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is impossible for this error
    /// to have been produced. Failure to do so is *undefined behavior* as
    /// calling this method makes promises to the compiler about what
    /// optimizations are valid.
    ///
    /// Proceed with caution.
    #[track_caller]
    #[cold]
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn panic_unchecked(self) -> ! {
        // SAFETY: The caller ensures it is impossible to reach this code.
        unsafe { S::KIND.0.handle_as_elems_error_unchecked(self.0) }
    }
}

impl<S> fmt::Debug for AsElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> fmt::Display for AsElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> Default for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S> Clone for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(
        &mut self,
        source: &Self,
    ) {
        self.0.clone_from(&source.0);
    }
}

impl<S> Copy for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Copy,
{
}

impl<S> PartialEq for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: PartialEq,
{
    #[inline]
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0 == other.0
    }
}

impl<S> Eq for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Eq,
{
}

impl<S> PartialOrd for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: PartialOrd,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Ord,
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> hash::Hash for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.0.hash(state);
    }
}

impl<S> core::error::Error for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: core::error::Error,
{
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0.description()
    }
}

#[cfg(target_pointer_width = "16")]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) type _OobIndex = i32;

#[cfg(target_pointer_width = "32")]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) type _OobIndex = i64;

#[cfg(target_pointer_width = "64")]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) type _OobIndex = i128;

/// A type for out-of-bounds errors.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OobIndex {
    /// SAFETY: This value is either *positive* or *negative*. It is *never zero*.
    ///
    ///         If it is negative, then it must fit inside a [`prim@isize`]. Otherwise,
    ///         if it is positive, then it must fit inside a [`prim@usize`].
    repr: NonZero<_OobIndex>,
}

impl OobIndex {
    /// The minimum supported index. This is equivalent to [`isize::MIN`].
    pub const MIN: OobIndex = OobIndex::from_negative(isize::MIN).unwrap();
    /// The maximum supported index. This is equivalent to [`usize::MIN`].
    pub const MAX: OobIndex = OobIndex::from_positive(usize::MAX).unwrap();

    /// Attempt to create a new [`OobIndex`] from a positive index.
    ///
    /// # Returns
    ///
    /// Returns `None` if `x` is not positive.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_positive(x: usize) -> Option<OobIndex> {
        if x > 0 {
            Some(OobIndex {
                repr: NonZero::new(x as _OobIndex).unwrap(),
            })
        } else {
            None
        }
    }

    /// Attempt to create a new [`OobIndex`] from a negative index.
    ///
    /// # Returns
    ///
    /// Returns `None` if `x` is not negative.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_negative(x: isize) -> Option<OobIndex> {
        if x < 0 {
            Some(OobIndex {
                repr: NonZero::new(x as _OobIndex).unwrap(),
            })
        } else {
            None
        }
    }

    /// Attempt to get the stored value as a positive number.
    ///
    /// # Returns
    ///
    /// Returns `Err(negative)` if `x` is not positive.
    #[inline(always)]
    #[track_caller]
    pub const fn as_positive(self) -> Result<NonZero<usize>, NonZero<isize>> {
        const NEG_START: _OobIndex = OobIndex::MIN.repr.get();
        // We want to use an exclusive range so we increment by 1.
        const POS_END: _OobIndex = OobIndex::MAX.repr.get() + 1;

        match self.repr.get() {
            // SAFETY: It is impossible for the value to be smaller than `isize::MIN`.
            ..NEG_START => unsafe { unreachable_unchecked!("`repr < isize::MIN`") },
            // NOTE: We're in the range of valid negative `isize`s.
            repr @ NEG_START..0 => Err(NonZero::new(repr as isize).unwrap()),
            // SAFETY: `repr` is nonzero.
            0 => unsafe { unreachable_unchecked!("`repr == 0`") },
            // NOTE: We're in the range of positive (non-zero) `usize`s.
            repr @ 1..POS_END => Ok(NonZero::new(repr as usize).unwrap()),
            // SAFETY: It is impossible for the value to be larger than `usize::MAX`.
            POS_END.. => unsafe { unreachable_unchecked!("`repr > usize::MAX`") },
        }
    }

    /// Attempt to get the stored value as a negative number.
    ///
    /// # Returns
    ///
    /// Returns `Err(positive)` is `x` is not negative.
    #[inline(always)]
    #[track_caller]
    pub const fn as_negative(self) -> Result<NonZero<isize>, NonZero<usize>> {
        match self.as_positive() {
            Ok(positive) => Err(positive),
            Err(negative) => Ok(negative),
        }
    }

    /// Returns whether this index is positive.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_positive(self) -> bool {
        // NOTE: This provides better codegen than `self.repr.is_positive()`.
        self.as_positive().is_ok()
    }

    /// Returns whether this index is negative.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_negative(self) -> bool {
        // NOTE: This provides better codegen than `self.repr.is_negative()`.
        self.as_negative().is_ok()
    }
}

impl fmt::Debug for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl fmt::Display for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl fmt::Binary for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl fmt::LowerHex for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl fmt::UpperHex for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl fmt::LowerExp for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl fmt::UpperExp for OobIndex {
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.repr.fmt(f)
    }
}

/// An error detailing why it is not possible to split some slice.
pub enum SplitError<S>
where
    S: Slice + ?Sized,
{
    /// The index is out of bounds of the length.
    OutOfBounds {
        /// The index that is out of bounds.
        index: NonZero<usize>,
        /// The length of the slice.
        len: usize,
    },
    /// Some other error occurred.
    Other(S::SplitErr),
}

impl<S> SplitError<S>
where
    S: Slice + ?Sized,
{
    /// Returns the index that caused this error.
    #[inline]
    #[must_use]
    pub const fn index(&self) -> Option<usize> {
        match self {
            // SplitError::OutOfBounds { index, .. } if index.get() <= (usize::MAX as OobIndex) => {
            //     todo!()
            // }
            SplitError::Other(error) => Some(S::KIND.0.split_error_index(error)),
            _ => None,
        }
    }

    /// Panics with an error message corresponding to this error.
    #[track_caller]
    #[cold]
    #[inline(never)]
    pub const fn panic(self) -> ! {
        match self {
            SplitError::OutOfBounds { .. } => panic!("index is out of bounds: `index >= len`"),
            SplitError::Other(error) => S::KIND.0.handle_split_error(error),
        }
    }

    /// Marks the code path that produced this error as impossible.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is impossible for this error
    /// to have been produced. Failure to do so is *undefined behavior* as
    /// calling this method makes promises to the compiler about what
    /// optimizations are valid.
    ///
    /// Proceed with caution.
    #[track_caller]
    #[cold]
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn panic_unchecked(self) -> ! {
        // SAFETY: The caller ensures it is imppossible to reach this code.
        match self {
            SplitError::OutOfBounds { .. } => unsafe {
                unreachable_unchecked!("index is out of bounds: `index >= len`")
            },
            SplitError::Other(error) => unsafe { S::KIND.0.handle_split_error_unchecked(error) },
        }
    }
}

impl<S> Clone for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match *self {
            Self::OutOfBounds { index, len } => Self::OutOfBounds { index, len },
            Self::Other(ref error) => Self::Other(error.clone()),
        }
    }
}

impl<S> Copy for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Copy,
{
}

impl<S> fmt::Debug for SplitError<S>
where
    S: Slice + ?Sized,
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::OutOfBounds { index, len } => f
                .debug_struct("OutOfBounds")
                .field("index", index)
                .field("len", len)
                .finish(),
            Self::Other(error) => f.debug_tuple("Other").field(error).finish(),
        }
    }
}

impl<S> fmt::Display for SplitError<S>
where
    S: Slice + ?Sized,
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            SplitError::OutOfBounds { index, len } => {
                core::write!(f, "index is out of bounds: `{index} >= {len}`")
            }
            SplitError::Other(other) => other.fmt(f),
        }
    }
}

impl<S> PartialEq for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: PartialEq,
{
    #[inline]
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (
                Self::OutOfBounds {
                    index: l_index,
                    len: l_len,
                },
                Self::OutOfBounds {
                    index: r_index,
                    len: r_len,
                },
            ) => l_index == r_index && l_len == r_len,
            (Self::Other(l), Self::Other(r)) => l == r,
            _ => false,
        }
    }
}

impl<S> Eq for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Eq,
{
}

impl<S> PartialOrd for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: PartialOrd,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        match (self, other) {
            (
                SplitError::OutOfBounds {
                    index: l_index,
                    len: l_len,
                },
                SplitError::OutOfBounds {
                    index: r_index,
                    len: r_len,
                },
            ) => Some({
                let index = l_index.cmp(r_index);
                let len = l_len.cmp(r_len);

                index.then(len)
            }),
            (SplitError::Other(l), SplitError::Other(r)) => l.partial_cmp(r),
            (SplitError::OutOfBounds { .. }, SplitError::Other(_)) => Some(Ordering::Greater),
            (SplitError::Other(_), SplitError::OutOfBounds { .. }) => Some(Ordering::Less),
        }
    }
}

impl<S> Ord for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Ord,
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        match (self, other) {
            (
                SplitError::OutOfBounds {
                    index: l_index,
                    len: l_len,
                },
                SplitError::OutOfBounds {
                    index: r_index,
                    len: r_len,
                },
            ) => {
                let index = l_index.cmp(r_index);
                let len = l_len.cmp(r_len);

                index.then(len)
            }
            (SplitError::Other(l), SplitError::Other(r)) => l.cmp(r),
            (SplitError::OutOfBounds { .. }, SplitError::Other(_)) => Ordering::Greater,
            (SplitError::Other(_), SplitError::OutOfBounds { .. }) => Ordering::Less,
        }
    }
}

impl<S> hash::Hash for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        mem::discriminant(self).hash(state);

        match self {
            SplitError::OutOfBounds { index, len } => {
                index.hash(state);
                len.hash(state);
            }
            SplitError::Other(error) => error.hash(state),
        }
    }
}

impl<S> core::error::Error for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: core::error::Error,
{
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            SplitError::OutOfBounds { .. } => None,
            SplitError::Other(error) => Some(error),
        }
    }

    #[allow(deprecated)]
    #[inline]
    fn description(&self) -> &str {
        match self {
            SplitError::OutOfBounds { .. } => "index is out of bounds: `index >= len`",
            SplitError::Other(error) => error.description(),
        }
    }
}

/// An error indicating why we failed to get something from a [`SplitError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FromSplitErrorFailure {
    /// The [`SplitError`] was not of the [`Other`](SplitError::Other) variant.
    NotOther,
}

impl fmt::Display for FromSplitErrorFailure {
    #[inline]
    #[allow(deprecated)]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl core::error::Error for FromSplitErrorFailure {
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            FromSplitErrorFailure::NotOther => "split error is not the `Other` variant",
        }
    }
}
