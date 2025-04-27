// use core::ptr::NonNull;

use super::{non_zst::NonZst, zst::Zst};

/// This helps us organize the stuff we need to do by whether or not
/// `T` is a ZST.
#[repr(C)]
pub(super) union Data<T> {
    /// Additional data needed for ZST slides.
    pub(super) zst: Zst<T>,
    /// Additional data neeeded for non-ZST slides.
    pub(super) non_zst: NonZst<T>,
}

impl<T> Data<T> {
    #[inline(always)]
    #[must_use]
    pub const fn borrow(&self) -> DataRef<'_, T> {
        match size_of::<T>() {
            0 => DataRef::Zst(unsafe { &self.zst }),
            1.. => DataRef::NonZst(unsafe { &self.non_zst }),
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn borrow_mut(&mut self) -> DataMut<'_, T> {
        match size_of::<T>() {
            0 => DataMut::Zst(unsafe { &mut self.zst }),
            1.. => DataMut::NonZst(unsafe { &mut self.non_zst }),
        }
    }
}

impl<T> Clone for Data<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Data<T> {}

pub(super) enum DataRef<'a, T> {
    Zst(&'a Zst<T>),
    NonZst(&'a NonZst<T>),
}

pub(super) enum DataMut<'a, T> {
    Zst(&'a mut Zst<T>),
    NonZst(&'a mut NonZst<T>),
}
