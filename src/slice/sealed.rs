use crate::marker::TypeEq;

/// The implementation of [`super::Slice`].
///
/// Downstream crates ***should never*** rely upon the implementation details of this module.
pub unsafe trait Sealed {
    /// What kind of "raw item"s are stored within this slice.
    ///
    /// Do note that just because a slice has this item type set to this,
    /// that does not imply all `[Self::Item]`s are valid instances of this
    /// slice type.
    type Item: Sized;

    /// A constant storing a type witness used for observers of this trait
    /// to implement polymorphism that works in const.
    const KIND: SliceWit<Self>;

    /// A constant denoting whether the item for this slice is a zero sized type.
    ///
    /// This is purely for convenience.
    const IS_ZST: bool = size_of::<Self::Item>() == 0;
}

pub struct SliceWit<S: super::Slice + ?Sized> {
    pub(crate) kind: SliceKind<S>,
}

pub(crate) enum SliceKind<S: super::Slice + ?Sized> {
    Slice(TypeEq<S, [S::Item]>),
}
