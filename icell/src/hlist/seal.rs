use super::{Cons, Nil};

pub trait ValidateId<I> {
    fn validate_id(&self, id: &I) -> bool;
}

pub trait IsUnique {
    fn is_unique(&self) -> bool;
}

pub trait CheckAddress {
    fn check_addr(&self, addr: usize) -> bool;
}

pub trait GetMutUnchecked<'a>: 'a {
    type Output: 'a;

    unsafe fn get_mut_unchecked(self) -> Self::Output;
}

#[inline(always)]
fn addrof<T: ?Sized>(ptr: &T) -> usize {
    ptr as *const T as *const () as usize
}

impl<T: ?Sized, R: ValidateId<I>, I: crate::Identifier> ValidateId<I>
    for Cons<&'_ crate::ICell<I::Id, T>, R>
{
    #[inline]
    fn validate_id(&self, id: &I) -> bool {
        let Cons(cell, ref rest) = *self;

        id.check_id(&cell.id()) && rest.validate_id(id)
    }
}

impl<I> ValidateId<I> for Nil {
    #[inline]
    fn validate_id(&self, _id: &I) -> bool {
        true
    }
}

impl<T: ?Sized, R: CheckAddress> CheckAddress for Cons<&'_ T, R> {
    #[inline]
    fn check_addr(&self, addr: usize) -> bool {
        let Cons(ptr, ref rest) = *self;

        addr != addrof(ptr) && rest.check_addr(addr)
    }
}

impl CheckAddress for Nil {
    #[inline]
    fn check_addr(&self, _addr: usize) -> bool {
        true
    }
}

impl<T: ?Sized, R: CheckAddress + IsUnique> IsUnique for Cons<&'_ T, R> {
    #[inline]
    fn is_unique(&self) -> bool {
        let Cons(ptr, ref rest) = *self;

        rest.check_addr(addrof(ptr)) && rest.is_unique()
    }
}

impl IsUnique for Nil {
    #[inline]
    fn is_unique(&self) -> bool {
        true
    }
}

impl<'a, I, T: ?Sized, R: GetMutUnchecked<'a>> GetMutUnchecked<'a>
    for Cons<&'a crate::ICell<I, T>, R>
{
    type Output = Cons<&'a mut T, R::Output>;

    #[inline]
    unsafe fn get_mut_unchecked(self) -> Self::Output {
        let Cons(cell, rest) = self;

        Cons(&mut *cell.as_ptr(), rest.get_mut_unchecked())
    }
}

impl GetMutUnchecked<'_> for Nil {
    type Output = Self;

    #[inline]
    unsafe fn get_mut_unchecked(self) -> Self {
        Self
    }
}
