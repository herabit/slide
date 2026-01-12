use core::ptr::NonNull;

use crate::{marker::TypeEq, slice::Slice};

/// The internal representation of a slide location.
#[repr(C)]
pub(crate) union Location<S>
where
    S: Slice + ?Sized,
{
    /// For when `S::Elem` is not a ZST, we work with a pointer offset from the start.
    offset_ptr: NonNull<S::Elem>,
    /// For when `S::Elem` is a ZST, we work with offsets.
    offset: usize,
}

impl<S> Location<S>
where
    S: Slice + ?Sized,
{
    /// An associated constant indicating whether we're working with ZSTs or not.
    pub(crate) const IS_ZST: bool = size_of::<S::Elem>() == 0;

    /// Create a new [`Location`].
    ///
    /// # Safety
    ///
    /// The caller must ensure `start.add(offset)` is within the bounds of the allocated object for `start`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new(
        start: NonNull<S::Elem>,
        offset: usize,
    ) -> Self {
        if Self::IS_ZST {
            Self { offset }
        } else {
            Self {
                // SAFETY: The caller ensures this is fine.
                offset_ptr: unsafe { start.add(offset) },
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
            OriginKind::Pointer(..) if Self::IS_ZST => Location { offset: 0 },
            // NOTE: We were provided a pointer and we're not dealing with ZSTs, so turn it into a location.
            OriginKind::Pointer(ptr) => Location {
                offset_ptr: ptr.coerce(origin),
            },
            // NOTE: We're already dealing with a location.
            OriginKind::Location(loc) => loc.coerce(origin),
        };

        if Self::IS_ZST {
            // SAFETY: The caller ensures this is fine.
            unsafe { self.offset.unchecked_sub(origin.offset) }
        } else {
            // SAFETY: The caller ensures this is fine.
            unsafe { self.offset_ptr.offset_from_unsigned(origin.offset_ptr) }
        }
    }

    /// Advance the location without checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is safe to increment this location
    /// by `amount`. Failure to do so is *undefined behavior*.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn advance(
        &mut self,
        amount: usize,
    ) {
        if Self::IS_ZST {
            // SAFETY: The caller ensures this is fine.
            unsafe { self.offset = self.offset.unchecked_add(amount) }
        } else {
            // SAFETY: The caller ensures this is fine.
            unsafe { self.offset_ptr = self.offset_ptr.add(amount) }
        }
    }

    /// Rewind the location without checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is safe to decrement this location
    /// by `amount`. Failure to do so is *undefined behavior*.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn rewind(
        &mut self,
        amount: usize,
    ) {
        if Self::IS_ZST {
            // SAFETY: The caller ensures this is sound.
            unsafe { self.offset = self.offset.unchecked_sub(amount) }
        } else {
            // SAFETY: The caller ensures this is sound.
            unsafe { self.offset_ptr = self.offset_ptr.sub(amount) }
        }
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
