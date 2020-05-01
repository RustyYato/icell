pub mod builder;

#[macro_export]
#[cfg(feature = "std")]
macro_rules! global_reuse {
    ($($v:vis type $name:ident($inner:ty));* $(;)?) => {$(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name;

        const _: () = {
            #[doc(hidden)]
            #[inline(always)]
            #[allow(non_snake_case, non_upper_case_globals)]
            unsafe fn __global_reuse__get_it() -> $crate::runtime::macros::Guard<$inner> {
                static ONCE__global_reuse: $crate::runtime::macros::Once =
                    $crate::runtime::macros::Once::new();
                static mut SET__global_reuse: $crate::runtime::macros::MaybeUninit<$crate::runtime::macros::Mutex<$inner>> =
                    $crate::runtime::macros::MaybeUninit::uninit();

                ONCE__global_reuse.call_once(|| SET__global_reuse = $crate::runtime::macros::MaybeUninit::new($crate::runtime::macros::Mutex::default()));

                let set = &*SET__global_reuse.as_ptr();
                set.lock().unwrap()
            }

            unsafe impl $crate::runtime::Reuse<$inner> for $name {
                fn put(&mut self, value: $inner) {
                    unsafe { __global_reuse__get_it().push(value) };
                }

                fn take(&mut self) -> Option<$inner> {
                    unsafe { __global_reuse__get_it().pop() }
                }
            }
        };
    )*};
}

#[cfg(feature = "std")]
pub mod macros {
    pub use std::mem::MaybeUninit;
    pub use std::sync::Once;
    pub type Mutex<T> = std::sync::Mutex<Vec<T>>;
    pub type Guard<T> = std::sync::MutexGuard<'static, Vec<T>>;
}

pub unsafe trait Counter: Eq + Copy {
    fn next() -> Self;

    fn try_next() -> Option<Self>;
}

pub unsafe trait Reuse<T> {
    fn put(&mut self, value: T);
    fn take(&mut self) -> Option<T>;
}

pub struct NoOpReuse;

#[cfg(feature = "std")]
global_reuse!(pub type GlobalReuse(SmallGlobal));

crate::runtime_id! {
    #[derive(Debug)] pub type Global([u8; 6]);
}

#[cfg(feature = "std")]
crate::runtime_id! {
    #[derive(Debug)]
    pub type SmallGlobal([u8; 4]);
}

#[cfg(feature = "std")]
pub type GlobalOwner = crate::Owner<Runtime<SmallGlobal, GlobalReuse>>;
pub type Owner<I = Global, R = NoOpReuse> = crate::Owner<Runtime<I, R>>;
pub type GlobalICell<T, I = SmallGlobal> = crate::ICell<RuntimeId<I>, T>;
pub type ICell<T, I = Global> = crate::ICell<RuntimeId<I>, T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeId<I>(I);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Runtime<I: Counter, R: Reuse<I> = NoOpReuse>(I, R);

impl<I> RuntimeId<I> {
    pub fn cell<T>(self, value: T) -> ICell<T, I> {
        unsafe { ICell::from_raw_parts(self, value) }
    }
}

unsafe impl<T> Reuse<T> for NoOpReuse {
    fn put(&mut self, _: T) {}
    fn take(&mut self) -> Option<T> {
        None
    }
}

impl Runtime<Global> {
    pub fn owner() -> Owner<Global> {
        Owner::new(Self::with_counter())
    }

    pub fn try_owner() -> Option<Owner<Global>> {
        Self::try_with_counter().map(Owner::new)
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Global::next(), NoOpReuse)
    }

    pub fn try_new() -> Option<Self> {
        Some(Self(Global::try_next()?, NoOpReuse))
    }
}

impl<I: Counter> Runtime<I> {
    pub fn owner_with_counter() -> Owner<I> {
        Owner::new(Self::with_counter())
    }

    pub fn try_owner_with_counter() -> Option<Owner<I>> {
        Self::try_with_counter().map(Owner::new)
    }

    #[allow(clippy::new_without_default)]
    pub fn with_counter() -> Self {
        Self(I::next(), NoOpReuse)
    }

    pub fn try_with_counter() -> Option<Self> {
        Some(Self(I::try_next()?, NoOpReuse))
    }
}

#[cfg(feature = "std")]
impl Runtime<SmallGlobal, GlobalReuse> {
    pub fn global_reuse_owner() -> GlobalOwner {
        Owner::new(Self::global_reuse_new())
    }

    pub fn try_global_reuse_owner() -> Option<GlobalOwner> {
        Self::try_global_reuse_new().map(Owner::new)
    }

    #[allow(clippy::new_without_default)]
    pub fn global_reuse_new() -> Self {
        Self::with_counter_and_reuse(GlobalReuse)
    }

    pub fn try_global_reuse_new() -> Option<Self> {
        Self::try_with_counter_and_reuse(GlobalReuse).ok()
    }
}

impl<I: Counter, R: Reuse<I>> Runtime<I, R> {
    pub fn owner_with_counter_and_reuse(reuse: R) -> Owner<I, R> {
        Owner::new(Self::with_counter_and_reuse(reuse))
    }

    pub fn try_owner_with_counter_and_reuse(reuse: R) -> Result<Owner<I, R>, R> {
        Self::try_with_counter_and_reuse(reuse).map(Owner::new)
    }

    pub fn with_counter_and_reuse(mut reuse: R) -> Self {
        Self(reuse.take().unwrap_or_else(I::next), reuse)
    }

    pub fn try_with_counter_and_reuse(mut reuse: R) -> Result<Self, R> {
        match reuse.take().or_else(I::try_next) {
            Some(id) => Ok(Self(id, reuse)),
            None => Err(reuse),
        }
    }
}

unsafe impl<I: Counter, R: Reuse<I>> crate::Identifier for Runtime<I, R> {
    type Id = RuntimeId<I>;

    fn id(&self) -> Self::Id {
        RuntimeId(self.0)
    }

    fn check_id(&self, &RuntimeId(id): &Self::Id) -> bool {
        self.0 == id
    }
}

impl<I: Counter, R: Reuse<I>> Drop for Runtime<I, R> {
    fn drop(&mut self) {
        let &mut Self(value, ref mut reuse) = self;

        reuse.put(value);
    }
}
