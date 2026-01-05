use core::{cmp::Ordering, fmt, hash, mem};

use crate::{macros::unreachable_unchecked, slice::Slice};

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
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[track_caller]
    #[cold]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> fmt::Display for FromElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn clone_from(&mut self, source: &Self) {
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
    fn eq(&self, other: &Self) -> bool {
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> hash::Hash for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[track_caller]
    #[cold]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> fmt::Display for AsElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<S> PartialEq for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> hash::Hash for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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

/// The error type that is returned when attempting to split a
/// slice.
///
/// ***TODO***
pub enum SplitError<S>
where
    S: Slice + ?Sized,
{
    /// Cannot split at the specified index, it is out of bounds.
    OutOfBounds {
        /// The index that is out of bounds.
        index: usize,
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
    /// Panics with an error message corresponding to this error.
    #[track_caller]
    #[cold]
    pub const fn panic(self) -> ! {
        match self {
            SplitError::OutOfBounds { .. } => panic!("index is out of bounds: `index >= len`"),
            SplitError::Other(other) => S::KIND.0.handle_split_error(other),
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
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[track_caller]
    #[cold]
    pub const unsafe fn panic_unchecked(self) -> ! {
        // SAFETY: The caller ensures it is imppossible to reach this code.
        match self {
            SplitError::OutOfBounds { .. } => unsafe {
                unreachable_unchecked!("index is out of bounds: `index >= len`")
            },
            SplitError::Other(other) => unsafe { S::KIND.0.handle_split_error_unchecked(other) },
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
            Self::Other(ref other) => Self::Other(other.clone()),
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { index, len } => f
                .debug_struct("OutOfBounds")
                .field("index", index)
                .field("len", len)
                .finish(),
            Self::Other(other) => f.debug_tuple("Other").field(other).finish(),
        }
    }
}

impl<S> fmt::Display for SplitError<S>
where
    S: Slice + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn eq(&self, other: &Self) -> bool {
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
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
    fn cmp(&self, other: &Self) -> Ordering {
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);

        match self {
            SplitError::OutOfBounds { index, len } => {
                index.hash(state);
                len.hash(state);
            }
            SplitError::Other(other) => other.hash(state),
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
            SplitError::Other(o) => Some(o),
        }
    }

    #[allow(deprecated)]
    #[inline]
    fn description(&self) -> &str {
        match self {
            SplitError::OutOfBounds { .. } => "index is out of bounds: `index >= len`",
            SplitError::Other(o) => o.description(),
        }
    }
}
