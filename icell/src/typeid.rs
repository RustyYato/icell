use core::marker::PhantomData;

pub use crate::make_anon_typeid_owner as new_anon;
pub use crate::make_typeid as make;

#[macro_export]
macro_rules! make_anon_typeid_owner {
    () => {
        unsafe { $crate::Owner::new($crate::typeid::Type::new_unchecked(|| ())) }
    };
}

#[macro_export]
macro_rules! make_typeid {
    ($($v:vis type $name:ident);* $(;)?) => {$(
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $v struct $name($crate::typeid::macros::UnsafeCreate);

        const _: () = {
            static __make_typeid_FLAG: $crate::typeid::macros::Flag = $crate::typeid::macros::Flag::new();

            impl $name {
                pub fn owner() -> $crate::typeid::Owner<Self> {
                    Self::try_owner().expect(concat!(
                        "attempted a reentrant acquire of a `Type<",
                        stringify!($name),
                        ">`"
                    ))
                }

                pub fn try_owner() -> Option<$crate::typeid::Owner<Self>> {
                    use $crate::typeid::{macros::UnsafeCreate, Type};

                    unsafe {
                        __make_typeid_FLAG.init();
                        if __make_typeid_FLAG.acquire() {
                            Some($crate::typeid::Owner::new(Type::new_unchecked(Self(UnsafeCreate::new()))))
                        } else {
                            None
                        }
                    }
                }
            }

            impl Drop for $name {
                fn drop(&mut self) {
                    unsafe {
                        __make_typeid_FLAG.release();
                    }
                }
            }
        };
    )*};
}

#[doc(hidden)]
pub mod macros {
    pub use core::cell::Cell;
    pub use flag::Flag;

    #[cfg(feature = "std")]
    pub use std::thread_local;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct UnsafeCreate(());

    impl UnsafeCreate {
        #[doc(hidden)]
        pub unsafe fn new() -> Self {
            Self(())
        }
    }

    #[cfg(feature = "std")]
    mod flag {
        use core::sync::atomic::{
            AtomicBool, AtomicUsize,
            Ordering::{Acquire, Relaxed},
        };
        use std::cell::UnsafeCell;
        use std::mem::MaybeUninit;
        use std::sync::{Condvar, Mutex, Once};

        thread_local! {
            static THREAD_ID: MaybeUninit<u8> = MaybeUninit::uninit();
        }

        fn get_thread_id() -> usize {
            THREAD_ID.with(|id| id.as_ptr() as usize)
        }

        pub struct Flag(
            Once,
            AtomicBool,
            AtomicUsize,
            UnsafeCell<MaybeUninit<Condvar>>,
            UnsafeCell<MaybeUninit<Mutex<()>>>,
        );

        unsafe impl Send for Flag {}
        unsafe impl Sync for Flag {}

        impl Flag {
            pub const fn new() -> Self {
                Self(
                    Once::new(),
                    AtomicBool::new(false),
                    AtomicUsize::new(0),
                    UnsafeCell::new(MaybeUninit::uninit()),
                    UnsafeCell::new(MaybeUninit::uninit()),
                )
            }

            pub fn init(&self) {
                let Self(once, _, _, cv, mx) = self;

                once.call_once(|| unsafe {
                    cv.get().write(MaybeUninit::new(Condvar::new()));
                    mx.get().write(MaybeUninit::new(Mutex::new(())));
                });
            }

            #[doc(hidden)]
            pub unsafe fn acquire(&self) -> bool {
                let Self(_, st, thread_id, _, _) = self;

                if st.compare_and_swap(false, true, Acquire) && !self.acquire_slow() {
                    return false;
                }

                thread_id.store(get_thread_id(), Relaxed);

                true
            }

            #[cold]
            #[doc(hidden)]
            pub unsafe fn acquire_slow(&self) -> bool {
                let Self(_, st, thread_id, cv, mx) = self;

                let cv = &*cv.get().cast::<Condvar>();
                let mx = &*mx.get().cast::<Mutex<()>>();

                if thread_id.load(Relaxed) == get_thread_id() {
                    // reentrant acquire, we can't make an owner if there is already
                    // an owner on the current thread, we also shouldn't block
                    // because we can never acquire an owner, so we must return false

                    false
                } else {
                    let _ = cv
                        .wait_while(mx.lock().unwrap(), |()| {
                            st.compare_and_swap(false, true, Relaxed)
                        })
                        .unwrap();

                    true
                }
            }

            #[doc(hidden)]
            #[allow(unused)]
            pub unsafe fn release(&self) {
                let Self(_, st, thread_id, cv, _) = self;

                let cv = &*cv.get().cast::<Condvar>();

                thread_id.store(0, Relaxed);
                st.store(false, Relaxed);

                // if we are running miri, we are single threaded, so we don't need to notify
                // also miri doesn't handle notify at all
                #[cfg(not(miri))]
                cv.notify_one();
            }
        }
    }

    #[cfg(not(feature = "std"))]
    mod flag {
        use core::sync::atomic::{AtomicBool, Ordering};

        pub struct Flag(AtomicBool);

        unsafe impl Send for Flag {}
        unsafe impl Sync for Flag {}

        impl Flag {
            pub const fn new() -> Self {
                Self(AtomicBool::new(false))
            }

            pub fn init(&self) {}

            #[doc(hidden)]
            pub fn acquire(&self) -> bool {
                const SPIN_LIMIT: u32 = 6;

                let mut backoff = 0;

                while self
                    .0
                    .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                    .is_err()
                {
                    for _ in 0..1 << backoff.min(SPIN_LIMIT) {
                        core::sync::atomic::spin_loop_hint();
                    }

                    if backoff <= SPIN_LIMIT {
                        backoff += 1;
                    }
                }

                true
            }

            #[doc(hidden)]
            pub fn release(&self) {
                self.0.store(false, Ordering::Release)
            }
        }
    }
}

struct Invariant<T>(fn() -> *mut T);

pub type Owner<Id> = crate::Owner<Type<Id>>;
pub type ICell<Id, T> = crate::ICell<TypeId<Id>, T>;

pub struct TypeId<T>(PhantomData<Invariant<T>>);
pub struct Type<T>(T, TypeId<T>);

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
    /// on the current process
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        Self(value, TypeId::new())
    }
}

unsafe impl<T> crate::Transparent for TypeId<T> {}
unsafe impl<T> crate::Identifier for Type<T> {
    type Id = TypeId<T>;

    fn id(&self) -> Self::Id {
        self.1
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
