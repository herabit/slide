#![allow(dead_code, unused_macros, unused_imports)]

/// Macro that tells the compiler that a code path is unreachable, and undefined behavior to reach.
macro_rules! unreachable_unchecked {
    ($first:tt $($rest:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            $crate::needs_unsafe(
                ::core::unreachable!(
                    ::core::concat!(
                        "undefined behavior: ",
                        $first
                    )

                    $($rest)*
                )
            )
        }

        #[cfg(not(debug_assertions))]
        {
            ::core::hint::unreachable_unchecked()
        }
    }};

    () => {
        $crate::macros::unrechable_unchecked!("unreachable_unchecked must never be reached")
    };
}

/// Macro that tells the compiler that it is undefined behavior for some condition
/// to be false.
macro_rules! assert_unchecked {
    ($cond:expr, $first:tt $($rest:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            $crate::needs_unsafe(
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
            ::core::hint::assert_unchecked($cond)
        }
    }};

    ($cond:expr $(,)?) => {
        $crate::macros::assert_unchecked!($cond, "condition is false")
    };
}

pub(crate) use assert_unchecked;
