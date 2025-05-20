#![allow(dead_code, unused_macros, unused_imports)]

/// Macro that tells the compiler that a code path is unreachable, and undefined behavior to reach.
macro_rules! unreachable_unchecked {
    ($first:tt $($rest:tt)*) => {{
        #[cfg(debug_assertions)]
        #[allow(unreachable_code)]
        {
            $crate::util::needs_unsafe(
                ::core::panic!(
                    ::core::concat!(
                        "undefined behavior: ",
                        $first
                    )

                    $($rest)*
                )
            )
        }

        #[cfg(not(debug_assertions))]
        #[allow(unreachable_code)]
        {
            let _ = ($first $($rest)*);
            ::core::hint::unreachable_unchecked()
        }
    }};

    () => {
        $crate::macros::unrechable_unchecked!("unreachable_unchecked must never be reached")
    };
}

pub(crate) use unreachable_unchecked;

/// Macro that tells the compiler that it is undefined behavior for some condition
/// to be false.
macro_rules! assert_unchecked {
    ($cond:expr, $first:tt $($rest:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            $crate::util::needs_unsafe(
                ::core::assert!(
                    $cond,
                    ::core::concat!(
                        "undefined behavior: ",
                        $first,
                    )
                    $($rest)*
                )
            )
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = ($first $($rest)*);
            ::core::hint::assert_unchecked($cond)
        }
    }};

    ($cond:expr $(,)?) => {
        $crate::macros::assert_unchecked!($cond, "condition is false")
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
