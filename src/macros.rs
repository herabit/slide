#![allow(dead_code, unused_macros, unused_imports)]

#[cfg(debug_assertions)]
#[doc(hidden)]
#[allow(unreachable_code)]
macro_rules! _unreachable_unchecked {
    ($first:tt $(, $($rest:tt)*)?) => {{
        #[allow(unreachable_code)]
        {
            $crate::util::needs_unsafe(
                ::core::panic!(
                    ::core::concat!(
                        "undefined behavior: ",
                        $first,
                    ),
                    $($($rest)*)?
                ),
            )
        }
    }};
}

#[cfg(not(debug_assertions))]
#[doc(hidden)]
macro_rules! _unreachable_unchecked {
    ($first:tt $(, $($rest:tt)*)?) => {
        $crate::util::__unreachable_unchecked(
            ::core::format_args!(
                ::core::concat!(
                    "undefined behavior: ",
                    $first,
                ),
                $($($rest)*)?
            ),
        )
    };
}

use core::fmt;

pub(crate) use _unreachable_unchecked;

/// Macro that tells the compiler that a code path is unreachable, and undefined behavior to reach.
macro_rules! unreachable_unchecked {
    ($first:tt $(, $($rest:tt)*)?) => {
        $crate::macros::_unreachable_unchecked!($first, $($($rest)*)?)
    };

    () => {
        $crate::macros::_unreachable_unchecked!("unreachable_unchecked must never be reached")
    };
}

pub(crate) use unreachable_unchecked;

#[cfg(debug_assertions)]
#[doc(hidden)]
macro_rules! _assert_unchecked {
    ($cond:expr, $first:tt $(, $($rest:tt)*)?) => {
        $crate::util::needs_unsafe(
            ::core::assert!(
                $cond,
                ::core::concat!(
                    "undefined behavior: ",
                    $first,
                ),
                $($($rest)*)?
            ),
        )
    };
}

#[cfg(not(debug_assertions))]
#[doc(hidden)]
macro_rules! _assert_unchecked {
    ($cond:expr, $first:tt $(, $($rest:tt)*)?) => {
        $crate::util::__assert_unchecked(
            $cond,
            ::core::format_args!(
                ::core::concat!(
                    "undefined behavior: ",
                    $first
                ),
                $($($rest)*)?
            ),
        )
    };
}

pub(crate) use _assert_unchecked;

/// Macro that tells the compiler that it is undefined behavior for some condition
/// to be false.
macro_rules! assert_unchecked {
    ($cond:expr, $first:tt $(, $($rest:tt)*)?) => {
        $crate::macros::_assert_unchecked!($cond, $first, $($($rest)*)?)
    };
    ($cond:expr $(,)?) => {
        $crate::macros::_assert_unchecked!($cond, "condition is false")
    };
}

pub(crate) use assert_unchecked;

/// Macro that proves that two types have the same size and alignment,
/// and that it is undefined behavior for them to differ.
macro_rules! assert_layout_unchecked {
    ($a:ty, $b:ty, $($arg:tt)+) => {
        $crate::macros::assert_unchecked!(
            const {
                ::core::mem::size_of::<$a>() == ::core::mem::size_of::<$b>()
                &&
                ::core::mem::align_of::<$a>() == ::core::mem::align_of::<$b>()
            },
            $($arg)*
        )
    };

    ($a:ty, $b:ty $(,)?) => {
        $crate::macros::assert_layout_unchecked!(
            $a,
            $b,
            "layout mismatch"
        )
    };
}

pub(crate) use assert_layout_unchecked;
