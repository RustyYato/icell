use core::marker::PhantomData;

pub use crate::make_scoped as owner;

#[macro_export]
macro_rules! make_scoped {
    ($name:ident) => {
        let $name = $crate::scoped::ScopedId::new();
        let enforce_unique_lifetime = $crate::scoped::EnforceUniqueLifetime(&$name);
        let $name = unsafe { $crate::scoped::Scoped::create_new_scoped($name) };
        let mut $name = $crate::Owner::new($name);
    };
}

pub fn with<F: FnOnce(Owner) -> R, R>(f: F) -> R {
    Scoped::with(f)
}

struct Invariant<'a>(&'a mut &'a ());

pub type Owner<'a> = crate::Owner<Scoped<'a>>;
pub type ICell<'a, T> = crate::ICell<ScopedId<'a>, T>;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopedId<'a>(PhantomData<Invariant<'a>>);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scoped<'a>(ScopedId<'a>);

#[doc(hidden)]
pub struct EnforceUniqueLifetime<'a>(pub &'a ScopedId<'a>);

impl Drop for EnforceUniqueLifetime<'_> {
    #[inline(always)]
    fn drop(&mut self) {}
}

impl<'a> ScopedId<'a> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a> Scoped<'a> {
    #[doc(hidden)]
    #[inline(always)]
    pub const unsafe fn create_new_scoped(id: ScopedId<'a>) -> Self {
        Self(id)
    }

    pub fn with<R, F: FnOnce(Owner) -> R>(f: F) -> R {
        unsafe { f(Owner::new(Self::create_new_scoped(ScopedId::new()))) }
    }
}

unsafe impl crate::Transparent for ScopedId<'_> {}
unsafe impl<'a> crate::Identifier for Scoped<'a> {
    type Id = ScopedId<'a>;

    fn id(&self) -> Self::Id {
        self.0
    }

    fn check_id(&self, _id: &Self::Id) -> bool {
        true
    }
}
