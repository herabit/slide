#![allow(dead_code)]

use crate::slice::{Slice, SliceWit};

/// Trait for sealing slice types.
pub trait Sealed {}

/// A public but internal implementation detail for working with slice types.
#[repr(transparent)]
pub struct SliceKind<S>(pub(crate) SliceWit<S>)
where
    S: Slice + ?Sized;

impl<S> Clone for SliceKind<S>
where
    S: Slice + ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for SliceKind<S> where S: Slice + ?Sized {}
