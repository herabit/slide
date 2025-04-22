#![cfg_attr(not(test), no_std)]

pub(crate) mod raw;
pub(crate) mod util;

pub(crate) mod slide;
#[doc(inline)]
pub use slide::*;

pub const LEFT: Direction = Direction::Left;
pub const RIGHT: Direction = Direction::Right;

/// An enum representing a direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Direction {
    #[default]
    Right = 1,
    Left = 0,
}

impl Direction {
    #[inline]
    #[must_use]
    pub const fn invert(self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_left(self) -> bool {
        matches!(self, Direction::Left)
    }

    #[inline]
    #[must_use]
    pub const fn is_right(self) -> bool {
        matches!(self, Direction::Right)
    }
}
