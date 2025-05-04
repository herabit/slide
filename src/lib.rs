#![cfg_attr(not(test), no_std)]

pub(crate) mod raw;
pub(crate) mod util;

// pub(crate) mod slide;
// #[doc(inline)]
// pub use slide::*;

// pub(crate) mod slide_mut;
// #[doc(inline)]
// pub use slide_mut::*;

/// Represents a direction in a slide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Dir {
    #[default]
    Ahead,
    Behind,
}
