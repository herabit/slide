#![allow(dead_code)]

use crate::util;
use core::ptr::NonNull;

/// A type for representing a position in some buffer.
///
/// - ZSTs: The position is represented as an offset from the start of the buffer.
/// - Non-ZSTs: The position is represented as a pointer within the buffer.
///
/// # Safety
///
/// - TODO: Write safety information.
#[repr(C)]
pub(crate) union Pos<T> {
    /// ZST representation.
    offset: usize,
    /// Non-ZST representation.
    ptr: NonNull<T>,
}

impl<T> Pos<T> {
    /// Create a new [`Pos`] from a given offset.
    ///
    /// # Panics
    ///
    /// - Panics if `T` is not zero-sized.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn from_offset(offset: usize) -> Self {
        assert!(size_of::<T>() == 0, "size must be zero");

        Self { offset }
    }

    /// Create a new [`Pos`] from a given pointer.
    ///
    /// # Panics
    ///
    /// - Panics if `T` is zero-sized.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn from_ptr(ptr: NonNull<T>) -> Self {
        assert!(size_of::<T>() != 0, "size must be nonzero");

        Self { ptr }
    }

    /// Create a new [`Pos`] from a given `start` and `offset`.
    ///
    /// # Safety
    ///
    /// - The memory range `start..start + offset` is contained within a
    ///   single, valid allocated object.
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn with_offset(start: NonNull<T>, offset: usize) -> Self {
        // SAFETY: The caller ensures this is fine.
        match size_of::<T>() {
            0 => Self::from_offset(offset),
            1.. => Self::from_ptr(unsafe { start.add(offset) }),
        }
    }

    /// Create a new [`Pos`] at pointing to `start`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn new(start: NonNull<T>) -> Self {
        // SAFETY: Offset zero is always in bounds for any allocated object.
        unsafe { Pos::with_offset(start, 0) }
    }

    /// Calculate the offset from `start` that this [`Pos`] represents.
    ///
    /// # Safety
    ///
    /// - `self` and `start` are derived from the same valid allocated object.
    ///
    /// - If `T` is not zero-sized, the caller must ensure the distance between
    ///   the two positions is nonnegative (`start <= self`).
    ///  
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn to_offset(self, start: NonNull<T>) -> usize {
        // SAFETY: The caller ensures this is fine.
        match size_of::<T>() {
            0 => unsafe { self.offset },
            1.. => unsafe { super::slice_len(start, self.ptr) },
        }
    }

    /// Calculate a pointer to this position given the `start` pointer.
    ///
    /// # Safety
    ///
    /// - TODO: Finish safety info.    
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn to_ptr(self, start: NonNull<T>) -> NonNull<T> {
        // SAFETY: The caller ensures this is fine.
        unsafe { start.add(self.to_offset(start)) }
    }

    /// Get this position whilst establishing it's relationship with the
    /// `start` pointer.
    ///
    /// # Safety
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn get(self, start: NonNull<T>) -> Self {
        // SAFETY: The caller ensures this is fine.
        unsafe { Self::with_offset(start, self.to_offset(start)) }
    }

    /// Calculate the offset between `self` and `start`.
    ///
    /// # Safety
    ///
    /// - If `T` is not zero-sized, `self` and `start` are
    ///   derived from the same valid allocated object.
    ///
    /// - The distance between the two positions is nonnegative (`start <= self`).
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn offset_from(self, start: Self) -> usize {
        // SAFETY: The caller ensures this is fine.
        match size_of::<T>() {
            0 => unsafe { self.offset.unchecked_sub(start.offset) },
            1.. => unsafe { super::slice_len(start.ptr, self.ptr) },
        }
    }

    /// Calculate a new position `n` elements ahead of `self` (`self + n`).
    ///
    /// # Safety
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn add(self, n: usize) -> Self {
        match size_of::<T>() {
            0 => Self::from_offset(unsafe { self.offset.unchecked_add(n) }),
            1.. => Self::from_ptr(unsafe { self.ptr.add(n) }),
        }
    }

    /// Calculate a new position `n` elements behind of `self` (`self - n`).
    ///
    /// # Safety
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn sub(self, n: usize) -> Self {
        match size_of::<T>() {
            0 => Self::from_offset(unsafe { self.offset.unchecked_sub(n) }),
            1.. => Self::from_ptr(unsafe { self.ptr.sub(n) }),
        }
    }
}

impl<T> Clone for Pos<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Pos<T> {}
