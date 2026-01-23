use core::ptr::NonNull;

use crate::{marker::TypeEq, slice::Slice};

/// The internal representation of a slide location.
#[repr(C)]
pub(crate) union Location<S>
where
    S: Slice + ?Sized,
{
    /// Variant for when we're working with pointer locations.
    ptr: NonNull<S::Elem>,
    /// Variant for when we're working with index locations.
    index: usize,
}

impl<S> Location<S>
where
    S: Slice + ?Sized,
{
    /// An associated constant used for determining whether we should use indices.
    ///
    /// We use indices if:
    ///
    /// - `S::Elem` is a zero sized type.
    ///
    /// - We're compiling with debug assertions. We do this to try to catch UB.
    ///
    /// - We're forced to.
    pub(crate) const INDEX_BASED: bool =
        size_of::<S::Elem>() == 0 || cfg!(debug_assertions) || cfg!(feature = "force_index");

    /// Create a new [`Location`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `start.add(offset)` is within the bounds of the same *allocated object* as `start`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new(
        start: NonNull<S::Elem>,
        offset: usize,
    ) -> Self {
        if Self::INDEX_BASED {
            Self { index: offset }
        } else {
            Self {
                // SAFETY: The caller ensures this is fine.
                ptr: unsafe { start.add(offset) },
            }
        }
    }

    /// Calculate the offset from `start` until `self`.
    ///
    /// # Safety
    ///
    /// - If `origin` is a `NonNull<S::Elem>` then it *must* come from the same
    ///   *allocated object* and exist at or before `self`.
    ///
    /// - If `origin` is a `Location<S>` then it *must* come from the same
    ///   *allocated object* and exist at or before `self`.
    ///
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn offset_from<O>(
        self,
        origin: O,
    ) -> usize
    where
        O: Origin<S>,
    {
        let origin = match O::KIND {
            // NOTE: Since we were given a pointer and we're dealing with ZSTs, we only want to return
            //       the offset of self, and `self - 0 == self`.
            OriginKind::Pointer(..) if Self::INDEX_BASED => Location { index: 0 },
            // NOTE: We were provided a pointer and we're not dealing with ZSTs, so turn it into a location.
            OriginKind::Pointer(ptr) => Location {
                ptr: ptr.coerce(origin),
            },
            // NOTE: We're already dealing with a location.
            OriginKind::Location(loc) => loc.coerce(origin),
        };

        if Self::INDEX_BASED {
            // SAFETY: The caller ensures this is fine.
            unsafe { self.index.unchecked_sub(origin.index) }
        } else {
            // SAFETY: The caller ensures this is fine.
            unsafe { self.ptr.offset_from_unsigned(origin.ptr) }
        }
    }

    /// Advance the location without checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is safe to increment this location
    /// by `amount`. Failure to do so is *undefined behavior*.
    ///
    /// # Returns
    ///
    /// Returns the advanced location.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn advance(
        self,
        amount: usize,
    ) -> Location<S> {
        if Self::INDEX_BASED {
            Location {
                // SAFETY: The caller ensures this is fine.
                index: unsafe { self.index.unchecked_add(amount) },
            }
        } else {
            Location {
                // SAFETY: The caller ensures this is fine.
                ptr: unsafe { self.ptr.add(amount) },
            }
        }
    }

    /// Advance the location without checks, in-place.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is safe to increment this location
    /// by `amount`. Failure to do so is *undefined behavior*.
    ///
    /// # Returns
    ///
    /// Returns the old location.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn advance_assign(
        &mut self,
        amount: usize,
    ) -> Location<S> {
        let old = *self;

        // SAFETY: The caller ensures this is fine.
        *self = unsafe { self.advance(amount) };

        old
    }

    /// Rewind the location without checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is safe to increment this location
    /// by `amount`. Failure to do so is *undefined behavior*.
    ///
    /// # Returns
    ///
    /// Returns the rewound location.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn rewind(
        self,
        amount: usize,
    ) -> Location<S> {
        if Self::INDEX_BASED {
            Location {
                // SAFETY: The caller ensures this is sound.
                index: unsafe { self.index.unchecked_sub(amount) },
            }
        } else {
            Location {
                // SAFETY: The caller ensures this is sound.
                ptr: unsafe { self.ptr.sub(amount) },
            }
        }
    }

    /// Rewind the location without checks, in-place.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is safe to decrement this location
    /// by `amount`. Failure to do so is *undefined behavior*.
    ///
    /// # Returns
    ///
    /// Returns the new location.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn rewind_assign(
        &mut self,
        amount: usize,
    ) -> Location<S> {
        // SAFETY: The caller ensures this is sound.
        *self = unsafe { self.rewind(amount) };

        *self
    }

    /// Apply this location to a given pointer.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that `origin` comes from the same
    /// *allocated object* as `self`, and exist at or before `self`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn apply(
        self,
        origin: NonNull<S::Elem>,
    ) -> NonNull<S::Elem> {
        // SAFETY: The caller ensures this is sound.
        let offset = unsafe { self.offset_from(origin) };

        // SAFETY: Since `offset` is valid, offsetting `origin` by `offset`
        //         is also, valid.
        unsafe { origin.add(offset) }
    }
}

impl<S> Clone for Location<S>
where
    S: Slice + ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Location<S> where S: Slice + ?Sized {}

/// Location offset schenanigans.
pub(crate) trait Origin<S>: Copy
where
    S: Slice + ?Sized,
{
    /// Type witness.
    #[doc(hidden)]
    const KIND: OriginKind<Self, S>;
}

impl<S> Origin<S> for Location<S>
where
    S: Slice + ?Sized,
{
    const KIND: OriginKind<Self, S> = OriginKind::Location(TypeEq::new());
}

impl<S> Origin<S> for NonNull<S::Elem>
where
    S: Slice + ?Sized,
{
    const KIND: OriginKind<Self, S> = OriginKind::Pointer(TypeEq::new());
}

/// Type witness for offsetting from a location.
pub(crate) enum OriginKind<O, S>
where
    S: Slice + ?Sized,
{
    /// Offset pointer location type.
    Pointer(TypeEq<O, NonNull<S::Elem>>),
    /// Offset location type.
    Location(TypeEq<O, Location<S>>),
}
