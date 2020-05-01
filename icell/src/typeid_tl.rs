use core::marker::PhantomData;

pub use crate::make_thread_local_typeid as make;

#[macro_export]
macro_rules! make_thread_local_typeid {
    ($($v:vis type $name:ident);* $(;)?) => {$(
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $v struct $name($crate::typeid::macros::UnsafeCreate);

        const _: () = {
            $crate::typeid::macros::thread_local!{
                static __make_thread_local_FLAG: $crate::typeid::macros::Cell<bool> = $crate::typeid::macros::Cell::new(false);
            }

            impl $name {
                pub fn try_owner() -> Option<$crate::typeid_tl::Owner<Self>> {
                    use $crate::typeid::macros::UnsafeCreate;
                    use $crate::typeid_tl::Type;

                    unsafe {
                        __make_thread_local_FLAG.with(|flag| {
                            if flag.get() {
                                None
                            } else {
                                flag.set(true);
                                Some($crate::typeid_tl::Owner::new(Type::new_unchecked(Self(UnsafeCreate::new()))))
                            }
                        })
                    }
                }

                pub fn owner() -> $crate::typeid_tl::Owner<Self> {
                    Self::try_owner().expect(concat!(
                        "attempted a reentrant acquire of a `Type<",
                        stringify!($name),
                        ">`"
                    ))
                }
            }

            impl Drop for $name {
                fn drop(&mut self) {
                    unsafe {
                        __make_thread_local_FLAG.with(|flag| flag.set(false));
                    }
                }
            }
        };
    )*};
}

struct NonThreadSafeInvariant<T>(*mut T);

pub type Owner<Id> = crate::Owner<Type<Id>>;
pub type ICell<Id, T> = crate::ICell<TypeId<Id>, T>;

pub struct TypeId<T>(PhantomData<NonThreadSafeInvariant<T>>);
pub struct Type<T>(T, PhantomData<NonThreadSafeInvariant<T>>);

unsafe impl<T> Send for TypeId<T> {}

impl<T> TypeId<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Type<T> {
    /// # Safety
    ///
    /// There must be at most 1 instance of `Type<T>` for any given `T`
    /// on the current thread
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        Self(value, PhantomData)
    }
}

unsafe impl<T> crate::Transparent for TypeId<T> {}
unsafe impl<T> crate::Identifier for Type<T> {
    type Id = TypeId<T>;

    fn id(&self) -> Self::Id {
        TypeId(self.1)
    }

    fn check_id(&self, _id: &Self::Id) -> bool {
        true
    }
}

impl<T> Default for TypeId<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Copy for TypeId<T> {}
impl<T> Clone for TypeId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> core::fmt::Debug for TypeId<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("TypeId")
            .field(&core::any::type_name::<T>())
            .finish()
    }
}

impl<T> core::fmt::Debug for Type<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Type")
            .field(&core::any::type_name::<T>())
            .finish()
    }
}

impl<T> Eq for TypeId<T> {}
impl<T> PartialEq for TypeId<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> PartialOrd for TypeId<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl<T> Ord for TypeId<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<T> core::hash::Hash for TypeId<T> {
    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
}

impl<T> Eq for Type<T> {}
impl<T> PartialEq for Type<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> PartialOrd for Type<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl<T> Ord for Type<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<T> core::hash::Hash for Type<T> {
    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
}
