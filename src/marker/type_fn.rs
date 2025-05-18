#![allow(type_alias_bounds)]

use core::{marker::PhantomData, ptr};

use crate::mem;

use super::variance::Covariant;

/// Marker trait for type-level functions.
///
/// # Properties
///
/// If `F: Func<A> + Func<B>`:
///
/// 1. If `A == B`, then `Call<F, A> == Call<F, B>`.
///
/// 2. If `Call<F, A> != Call<F, B>`, then `A != B`.
pub(crate) trait Func<Arg: ?Sized> {
    type Output: ?Sized;
}

/// Marker trait for type-level functions that can be reversed.
///
/// # Properties
///
/// If `F: RevFunc<A> + RevFunc<A>`:
///
/// 1. If `A == B`, then `Uncall<F, A> == Uncall<F, B>`.
///
/// 2. If `A != B`, then `Uncall<F, A> != Uncall<F, B>`.
pub(crate) trait RevFunc<Ret: ?Sized>: Func<Self::Arg, Output = Ret> {
    type Arg: ?Sized;
}

/// Marker trait for type-level functions that are injective.
///
/// # Properties
///
/// If `F: InjFunc<A> + InjFunc<B>`:
///
/// 1. If `A == B`, then `Call<F, A> == Call<F, B>`.
///
/// 2. If `Call<F, A> == Call<F, B>`, then `A == B`.
///
/// 3. If `A != B`, then `Call<F, A> != Call<F, B>`.
///
/// 4. If `Call<F, A> != Call<F, B>`, then `A != B`.
pub(crate) trait InjFunc<Arg: ?Sized>:
    Func<Arg, Output = Self::Ret> + RevFunc<Self::Ret, Arg = Arg>
{
    type Ret: ?Sized;
}

/// Marker trait for types with an associated type-level function
/// that creates them.
pub(crate) trait HasFunc {
    /// The argument that is used to create `Self` with `Self::Func`.
    type Arg: ?Sized;

    /// The type-level function that creates `Self` provided an `Self::Arg`.
    type Func: ?Sized + InjFunc<Self::Arg, Ret = Self>;
}

// /// Marker trait for types that are created from the same type-level function.
// pub(crate) trait SameFunc<T: ?Sized + HasFunc>: HasFunc<Func = T::Func> {}

// impl<T: ?Sized, U: ?Sized> SameFunc<U> for T
// where
//     T: HasFunc,
//     U: HasFunc<Func = T::Func>,
// {
// }

impl<F: ?Sized, A: ?Sized, R: ?Sized> InjFunc<A> for F
where
    F: Func<A, Output = R> + RevFunc<R, Arg = A>,
{
    type Ret = R;
}

/// Calls a type level function on a provided type.
pub(crate) type Call<F: Func<A> + ?Sized, A: ?Sized> = <F as Func<A>>::Output;

/// Reverses a type level function on a provided type.
pub(crate) type Uncall<F: RevFunc<R> + ?Sized, R: ?Sized> = <F as RevFunc<R>>::Arg;

/// A type-level function that reverses another.
pub(crate) struct Rev<F: ?Sized>(Covariant<F>);

// The empty tuple is the identity element.
impl<A: ?Sized> Func<A> for () {
    type Output = A;
}

impl<A: ?Sized> RevFunc<A> for () {
    type Arg = A;
}

// A [`PhantomData`] for an `F` is just `F`.
impl<F: ?Sized, A: ?Sized> Func<A> for PhantomData<F>
where
    F: Func<A>,
{
    type Output = Call<F, A>;
}

impl<F: ?Sized, R: ?Sized> RevFunc<R> for PhantomData<F>
where
    F: RevFunc<R>,
{
    type Arg = Uncall<F, R>;
}

impl<F: ?Sized> Rev<F> {
    /// Creates a new [`Rev`] for `F`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(Covariant::new())
    }
}

impl<F: ?Sized, A: ?Sized> Func<A> for Rev<F>
where
    F: RevFunc<A>,
{
    type Output = Uncall<F, A>;
}

impl<F: ?Sized, R: ?Sized> RevFunc<R> for Rev<F>
where
    F: InjFunc<R>,
{
    type Arg = <F as InjFunc<R>>::Ret;
}

impl<F: ?Sized> Clone for Rev<F> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<F: ?Sized> Copy for Rev<F> {}

/// A type-level function that wraps a given type
/// in a pointer.
#[derive(Clone, Copy)]
pub(crate) struct Ptr;

impl<A: ?Sized> Func<A> for Ptr {
    type Output = *const A;
}

impl<A: ?Sized> RevFunc<*const A> for Ptr {
    type Arg = A;
}

impl<A: ?Sized> HasFunc for *const A {
    type Arg = A;
    type Func = Ptr;
}

/// A type-level function that wraps a given type
/// in a mutable pointer.
#[derive(Clone, Copy)]
pub(crate) struct PtrMut;

impl<A: ?Sized> Func<A> for PtrMut {
    type Output = *mut A;
}

impl<A: ?Sized> RevFunc<*mut A> for PtrMut {
    type Arg = A;
}

impl<A: ?Sized> HasFunc for *mut A {
    type Arg = A;
    type Func = PtrMut;
}

/// A type-level function that wraps a given type
/// in a [`ptr::NonNull`].
#[derive(Clone, Copy)]
pub(crate) struct NonNull;

impl<A: ?Sized> Func<A> for NonNull {
    type Output = ptr::NonNull<A>;
}

impl<A: ?Sized> RevFunc<ptr::NonNull<A>> for NonNull {
    type Arg = A;
}

impl<A: ?Sized> HasFunc for ptr::NonNull<A> {
    type Arg = A;
    type Func = NonNull;
}

/// A type-level function that wraps a given type
/// in a reference.
#[derive(Clone, Copy)]
pub(crate) struct Ref<'a>(Covariant<&'a ()>);

impl<'a> Ref<'a> {
    /// Create a new [`Ref`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(Covariant::new())
    }
}

impl<'a, A: ?Sized + 'a> Func<A> for Ref<'a> {
    type Output = &'a A;
}

impl<'a, A: ?Sized + 'a> RevFunc<&'a A> for Ref<'a> {
    type Arg = A;
}

impl<'a, A: ?Sized + 'a> HasFunc for &'a A {
    type Arg = A;
    type Func = Ref<'a>;
}

/// A type-level function that wraps a given type
/// in a mutable reference.
#[derive(Clone, Copy)]
pub(crate) struct RefMut<'a>(Covariant<&'a mut ()>);

impl<'a> RefMut<'a> {
    /// Create a new [`RefMut`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(Covariant::new())
    }
}

impl<'a, A: ?Sized + 'a> Func<A> for RefMut<'a> {
    type Output = &'a mut A;
}

impl<'a, A: ?Sized + 'a> RevFunc<&'a mut A> for RefMut<'a> {
    type Arg = A;
}

impl<'a, A: ?Sized + 'a> HasFunc for &'a mut A {
    type Arg = A;
    type Func = RefMut<'a>;
}

/// A type-level function that wraps a given type in a slice.
#[derive(Clone, Copy)]
pub(crate) struct Slice;

impl<A> Func<A> for Slice {
    type Output = [A];
}

impl<A> RevFunc<[A]> for Slice {
    type Arg = A;
}

impl<A> HasFunc for [A] {
    type Arg = A;
    type Func = Slice;
}

/// A type-level function that wraps a given type in an [`Option`].
#[derive(Clone, Copy)]
pub(crate) struct Opt;

impl<A> Func<A> for Opt {
    type Output = Option<A>;
}

impl<A> RevFunc<Option<A>> for Opt {
    type Arg = A;
}

impl<A> HasFunc for Option<A> {
    type Arg = A;
    type Func = Opt;
}

/// A type-level function that creates a [`Result`] given a tuple.
#[derive(Clone, Copy)]
pub(crate) struct Res;

impl<T, E> Func<(T, E)> for Res {
    type Output = Result<T, E>;
}

impl<T, E> RevFunc<Result<T, E>> for Res {
    type Arg = (T, E);
}

impl<T, E> HasFunc for Result<T, E> {
    type Arg = (T, E);
    type Func = Res;
}

/// A type-level function that wraps a given type in a [`mem::NoDrop`].
#[derive(Clone, Copy)]
pub(crate) struct NoDrop;

impl<A: ?Sized> Func<A> for NoDrop {
    type Output = mem::NoDrop<A>;
}

impl<A: ?Sized> RevFunc<mem::NoDrop<A>> for NoDrop {
    type Arg = A;
}

impl<A: ?Sized> HasFunc for mem::NoDrop<A> {
    type Arg = A;
    type Func = NoDrop;
}

/// A type-level function that wraps a given type in a [`alloc::boxed::Box`].
#[cfg(feature = "alloc")]
pub(crate) struct Box;

#[cfg(feature = "alloc")]
impl<A: ?Sized> Func<A> for Box {
    type Output = alloc::boxed::Box<A>;
}

#[cfg(feature = "alloc")]
impl<A: ?Sized> RevFunc<alloc::boxed::Box<A>> for Box {
    type Arg = A;
}

#[cfg(feature = "alloc")]
impl<A: ?Sized> HasFunc for alloc::boxed::Box<A> {
    type Arg = A;
    type Func = Box;
}

/// A type-level function that wraps a given type in a [`alloc::rc::Rc`].
#[cfg(feature = "alloc")]
pub(crate) struct Rc;

#[cfg(feature = "alloc")]
impl<A: ?Sized> Func<A> for Rc {
    type Output = alloc::rc::Rc<A>;
}

#[cfg(feature = "alloc")]
impl<A: ?Sized> RevFunc<alloc::rc::Rc<A>> for Rc {
    type Arg = A;
}

#[cfg(feature = "alloc")]
impl<A: ?Sized> HasFunc for alloc::rc::Rc<A> {
    type Arg = A;
    type Func = Rc;
}

/// A type-level function that wraps a given type in a [`alloc::sync::Arc`].
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub(crate) struct Arc;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
impl<A: ?Sized> Func<A> for Arc {
    type Output = alloc::sync::Arc<A>;
}

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
impl<A: ?Sized> RevFunc<alloc::sync::Arc<A>> for Arc {
    type Arg = A;
}

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
impl<A: ?Sized> HasFunc for alloc::sync::Arc<A> {
    type Arg = A;
    type Func = Arc;
}
