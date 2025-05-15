#![allow(dead_code, unused_macros, unused_imports)]

/// Macro that tells the compiler that a code path is unreachable, and undefined behavior to reach.
macro_rules! unreachable_unchecked {
    ($first:tt $($rest:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            $crate::util::needs_unsafe(
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
            ::core::hint::assert_unchecked($cond)
        }
    }};

    ($cond:expr $(,)?) => {
        $crate::macros::assert_unchecked!($cond, "condition is false")
    };
}

pub(crate) use assert_unchecked;

// /// Macro that allows you to loop over the elements in a slice in const.
// macro_rules! slice_iter {
//     ($slice:expr, |$elem:ident $(, $label:lifetime)? $(,)?| $block:expr) => {{
//         #[inline(always)]
//         #[must_use]
//         #[track_caller]
//         const fn _not_zst<T>(x: &[T]) -> &[T] {
//             ::core::assert!(size_of::<T>() != 0, "size must be nonzero");

//             x
//         }

//         let slice = _not_zst($slice);
//     }};
// }

// pub(crate) use slice_iter;
