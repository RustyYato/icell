use crate::hlist::*;
use core::cell::UnsafeCell;

/// Represents a unique identifier that can be used by an `Owner` to recognize an `ICell`
///
/// # Safety
///
/// Given two instances of `a, b: Identifier`, the following proposition must hold:
///
/// Forall instances of `Identifier` `a` and `b`, if `Identifier::check_id(&a) && Identifier::check_id(&b)`
/// and `a` is alive on the same thread as `b` then an exclusive/shared borrow of `a`, must also exclusively/shared
/// borrow `b` at least as long as `a` (respectively)
pub unsafe trait Identifier {
    /// The ICell identifier that this `Identifier` recognizes.
    type Id;

    /// Get an instance in `Self::Id` that `check_id` will recognize.
    fn id(&self) -> Self::Id;

    /// check if the given `Self::Id` matches the given `Identifier`
    fn check_id(&self, id: &Self::Id) -> bool;
}

/// Represents an `Identifier::Id` that can be forged from the void.
///
/// # Safety
///
/// If `size_of::<Self>() == 0`, then it must be safe to forge instances of`<Self as Identifier>::Id`
pub unsafe trait Transparent: Copy {}

/// An owner that can be used to access the contents of an `ICell`
#[repr(transparent)]
pub struct Owner<I> {
    ident: I,
}

/// A sort of `Cell` that must be access with an `Owner`
#[repr(C)]
pub struct ICell<Id, T: ?Sized> {
    id: Id,
    value: UnsafeCell<T>,
}

unsafe impl<Id: Sync, T: Send + Sync> Sync for ICell<Id, T> {}

impl<O> Owner<O> {
    pub const fn new(ident: O) -> Self {
        Self { ident }
    }
}

impl<I: Identifier> Owner<I> {
    pub fn cell<T>(&self, value: T) -> ICell<I::Id, T> {
        unsafe { ICell::from_raw_parts(self.ident.id(), value) }
    }

    pub fn read<'a, T: ?Sized>(&'a self, cell: &'a ICell<I::Id, T>) -> &'a T {
        assert!(
            self.ident.check_id(cell.id()),
            "Tried to read using an unrelated owner"
        );

        unsafe { &*cell.as_ptr() }
    }

    pub fn swap<T>(&mut self, a: &ICell<I::Id, T>, b: &ICell<I::Id, T>) {
        assert!(
            self.ident.check_id(a.id()) && self.ident.check_id(b.id()),
            "Tried to swap using an unrelated owner"
        );

        unsafe { a.as_ptr().swap(b.as_ptr()) }
    }

    pub fn write<'a, T: ?Sized>(&'a mut self, cell: &'a ICell<I::Id, T>) -> &'a mut T {
        assert!(
            self.ident.check_id(cell.id()),
            "Tried to write using an unrelated owner"
        );

        unsafe { &mut *cell.as_ptr() }
    }

    pub fn write_2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        a: &'a ICell<I::Id, T>,
        b: &'a ICell<I::Id, U>,
    ) -> (&'a mut T, &'a mut U) {
        write_all!(self => a, b)
    }

    pub fn write_3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        a: &'a ICell<I::Id, T>,
        b: &'a ICell<I::Id, U>,
        c: &'a ICell<I::Id, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        write_all!(self => a, b, c)
    }

    pub fn write_4<'a, T: ?Sized, U: ?Sized, V: ?Sized, W: ?Sized>(
        &'a mut self,
        a: &'a ICell<I::Id, T>,
        b: &'a ICell<I::Id, U>,
        c: &'a ICell<I::Id, V>,
        d: &'a ICell<I::Id, W>,
    ) -> (&'a mut T, &'a mut U, &'a mut V, &'a mut W) {
        write_all!(self => a, b, c, d)
    }

    #[doc(hidden)]
    pub fn write_all<'a, C>(&'a mut self, cells: C) -> C::Output
    where
        C: GetMut<'a, I>,
    {
        cells.get_mut(&self.ident)
    }
}

impl<Id, T> ICell<Id, T> {
    pub const unsafe fn from_raw_parts(id: Id, value: T) -> Self {
        Self {
            id,
            value: UnsafeCell::new(value),
        }
    }

    pub fn retag<NewId>(self, new_id: NewId) -> ICell<NewId, T> {
        ICell {
            id: new_id,
            value: self.value,
        }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<Id, T: ?Sized> ICell<Id, T> {
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    pub const fn id(&self) -> &Id {
        &self.id
    }
}

impl<Id: Transparent, T> ICell<Id, T> {
    pub fn new(value: T) -> Self {
        assert_eq!(core::mem::size_of::<Id>(), 0);

        // Safety
        //
        // this is safe because `Id: Transparent`, meaning there are no
        // safety constraints on creating an `Id`
        #[allow(deprecated)]
        unsafe {
            let id = core::ptr::read(core::mem::align_of::<Id>() as *const Id);
            Self::from_raw_parts(id, value)
        }
    }
}

impl<Id: Transparent, T> ICell<Id, [T]> {
    pub fn as_slice_of_cells(&self) -> &[ICell<Id, T>] {
        assert_eq!(core::mem::size_of::<Id>(), 0);

        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe {
            let ptr = self.value.get();
            let len = (*ptr).len();
            let ptr = ptr.cast::<ICell<Id, T>>();
            core::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn as_slice_of_cells_mut(&mut self) -> &mut [ICell<Id, T>] {
        assert_eq!(core::mem::size_of::<Id>(), 0);

        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe {
            let ptr = self.value.get();
            let len = (*ptr).len();
            let ptr = ptr.cast::<ICell<Id, T>>();
            core::slice::from_raw_parts_mut(ptr, len)
        }
    }
}

impl<Id: Transparent, T: ?Sized> ICell<Id, T> {
    pub fn from_mut(value: &mut T) -> &mut Self {
        assert_eq!(core::mem::size_of::<Id>(), 0);

        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe {
            core::mem::transmute(value)
        }
    }
}

impl<Id: Transparent, T> ICell<Id, [T]> {
    pub fn transpose(value: &mut Self) -> &mut [ICell<Id, T>] {
        assert_eq!(core::mem::size_of::<Id>(), 0);

        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe {
            core::mem::transmute(value)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        assert_eq!(core::mem::size_of::<Id>(), 0);

        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe {
            core::mem::transmute(self)
        }
    }
}
