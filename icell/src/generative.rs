use core::marker::PhantomData;

pub use crate::make_generative as new;

#[macro_export]
macro_rules! make_generative {
    ($name:ident) => {
        let $name = $crate::generative::GenerativeId::new();
        let enforce_unique_lifetime = $crate::generative::EnforceUniqueLifetime(&$name);
        let $name = unsafe { $crate::generative::Generative::create_new_generative($name) };
        let mut $name = $crate::Owner::new($name);
    };
}

struct Invariant<'a>(&'a mut &'a ());

pub type Owner<'a> = crate::Owner<Generative<'a>>;
pub type ICell<'a, T> = crate::ICell<GenerativeId<'a>, T>;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenerativeId<'a>(PhantomData<Invariant<'a>>);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Generative<'a>(GenerativeId<'a>);

#[doc(hidden)]
pub struct EnforceUniqueLifetime<'a>(pub &'a GenerativeId<'a>);

impl Drop for EnforceUniqueLifetime<'_> {
    #[inline(always)]
    fn drop(&mut self) {}
}

impl<'a> GenerativeId<'a> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a> Generative<'a> {
    #[doc(hidden)]
    #[inline(always)]
    pub const unsafe fn create_new_generative(id: GenerativeId<'a>) -> Self {
        Self(id)
    }

    pub fn with<R, F: FnOnce(Owner) -> R>(f: F) -> R {
        unsafe { f(Owner::new(Self::create_new_generative(GenerativeId::new()))) }
    }
}

unsafe impl crate::Transparent for GenerativeId<'_> {}
unsafe impl<'a> crate::Identifier for Generative<'a> {
    type Id = GenerativeId<'a>;

    fn id(&self) -> Self::Id {
        self.0
    }

    fn check_id(&self, _id: &Self::Id) -> bool {
        true
    }
}
