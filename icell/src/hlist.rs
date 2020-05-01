pub struct Cons<T, N>(pub T, pub N);
pub struct Nil;

#[doc(hidden)]
#[macro_export]
macro_rules! hlist {
    () => {
        $crate::hlist::Nil
    };
    ($first:expr $(, $rest:expr)* $(,)?) => {
        $crate::hlist::Cons($first, $crate::hlist!($($rest),*))
    };
}

mod seal;

pub trait GetMut<'a, I> {
    type Output;

    fn get_mut(self, owner: &I) -> Self::Output;
}

impl<'a, T: ?Sized, R, I: crate::Identifier> GetMut<'a, I> for Cons<&'a crate::ICell<I::Id, T>, R>
where
    Self: seal::ValidateId<I> + seal::IsUnique + seal::GetMutUnchecked<'a>,
{
    type Output = <Self as seal::GetMutUnchecked<'a>>::Output;

    fn get_mut(self, id: &I) -> Self::Output {
        use seal::*;

        assert!(
            self.validate_id(id),
            "Tried to access a cell with an unrelated owner"
        );

        assert!(
            self.is_unique(),
            "Tried to obtain write access to aliasing cells"
        );

        unsafe { self.get_mut_unchecked() }
    }
}

impl<I> GetMut<'_, I> for Nil {
    type Output = Self;

    fn get_mut(self, _id: &I) -> Self {
        Self
    }
}
