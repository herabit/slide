use core::ptr::NonNull;

use crate::util::assert_unchecked;

use super::Start;

/// A position relative to some [`Start`].
#[repr(C)]
pub(crate) union Pos<T> {
    /// A position for a slide over ZSTs is simply it's offset from the start.
    zst: usize,
    /// A position for a slide over Non-ZSTs is a pointer *after* the start.
    non_zst: NonNull<T>,
}

impl<T> Pos<T> {
    /// Create a new [`Pos`] from `start + offset`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new(start: Start<T>, offset: usize) -> Pos<T> {
        match size_of::<T>() {
            0 => Pos { zst: offset },
            1.. => Pos {
                non_zst: unsafe { start.0.add(offset) },
            },
        }
    }

    /// Determine the offset of `self` relative to `start`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `start <= self` if `T` is not zero sized.
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn offset_from_start(self, start: Start<T>) -> usize {
        match size_of::<T>() {
            0 => unsafe { self.zst },
            1.. => unsafe {
                let offset = self.non_zst.offset_from(start.0);

                assert_unchecked!(offset >= 0, "undefined behavior: `start > self`");

                offset as usize
            },
        }
    }

    /// Determine the offset of `self` relative to `origin`.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - That `origin <= self`.
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn offset_from(self, origin: Self) -> usize {
        match size_of::<T>() {
            0 => unsafe {
                assert_unchecked!(
                    origin.zst <= self.zst,
                    "undefined behavior: `origin > self`"
                );

                self.zst.unchecked_sub(origin.zst)
            },
            1.. => unsafe {
                let offset = self.non_zst.offset_from(origin.non_zst);

                assert_unchecked!(offset >= 0, "undefined behavior: `origin > self`");

                offset as usize
            },
        }
    }

    /// Increment the position by `n` elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn add(self, n: usize) -> Self {
        match size_of::<T>() {
            0 => Self {
                zst: unsafe { self.zst.unchecked_add(n) },
            },
            1.. => Self {
                non_zst: unsafe { self.non_zst.add(n) },
            },
        }
    }

    /// Decrement the position by `n` elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - TODO: Finish safety info.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn sub(self, n: usize) -> Self {
        match size_of::<T>() {
            0 => Self {
                zst: unsafe { self.zst.unchecked_sub(n) },
            },
            1.. => Self {
                non_zst: unsafe { self.non_zst.sub(n) },
            },
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
