#![allow(dead_code)]

use core::ops::RangeBounds;

/// Implementation details.
pub(crate) mod private;

/// Module for the [`SliceRange`] type.
mod slice_range;

#[doc(inline)]
pub use slice_range::{SliceRange, SliceRangeError};

pub unsafe trait SliceBounds: private::Sealed + RangeBounds<usize> {}
