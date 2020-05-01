#![allow(clippy::many_single_char_names)]

use core::sync::atomic::*;
#[doc(hidden)]
pub use core::sync::atomic::{spin_loop_hint, Ordering};

#[macro_export]
macro_rules! runtime_id {
    ($($(#[$meta:meta])* $v:vis type $ident:ident($inner:ty));* $(;)?) => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $v struct $ident($inner);

        impl $ident {
            pub fn owner() -> $crate::runtime::Owner<Self> {
                $crate::runtime::Runtime::owner_with_counter()
            }

            pub fn try_owner() -> Option<$crate::runtime::Owner<Self>> {
                $crate::runtime::Runtime::try_owner_with_counter()
            }
        }

        unsafe impl $crate::runtime::Counter for $ident {
            fn next() -> Self {
                <Self as $crate::runtime::Counter>::try_next().expect("Tried to overflow ids!")
            }

            #[allow(non_camel_case_types, non_upper_case_globals)]
            fn try_next() -> Option<Self> {
                type ScalarAtomic__runtime_id = <$inner as $crate::runtime::builder::Scalar>::Atomic;
                static GLOBAL__runtime_id: ScalarAtomic__runtime_id = $crate::runtime::builder::Atomic::INIT;

                let value = GLOBAL__runtime_id.load($crate::runtime::builder::Ordering::Relaxed);

                loop {
                    let next = <ScalarAtomic__runtime_id as $crate::runtime::builder::Atomic>::next(value)?;
                    let id = $crate::runtime::builder::Scalar::try_from_atomic_inner(value).ok()?;

                    if GLOBAL__runtime_id.compare_exchange_weak(value, next, $crate::runtime::builder::Ordering::Relaxed, $crate::runtime::builder::Ordering::Relaxed).is_ok() {
                        return Some($ident(id))
                    }

                    $crate::runtime::builder::spin_loop_hint()
                }
            }
        }
    )*};
}

pub unsafe trait Atomic {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self;
    type Value;

    fn next(value: Self::Value) -> Option<Self::Value>;
}

pub unsafe trait Scalar: Copy {
    type Atomic: Atomic;
    type Error;

    fn try_from_atomic_inner(atomic: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error>;
}

macro_rules! atomic {
    ($($atomic:ident($local:ident))*) => {$(
        unsafe impl Atomic for $atomic {
            const INIT: Self = $atomic::new($local::MIN);
            type Value = $local;

            fn next(value: Self::Value) -> Option<Self::Value> {
                value.checked_add(1)
            }
        }

        unsafe impl Scalar for $local {
            type Atomic = $atomic;
            type Error = core::convert::Infallible;

            fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
                Ok(value)
            }
        }
    )*}
}

atomic! {
    AtomicU8(u8)
    AtomicU16(u16)
    AtomicU32(u32)
    AtomicU64(u64)
    AtomicUsize(usize)
    AtomicI8(i8)
    AtomicI16(i16)
    AtomicI32(i32)
    AtomicI64(i64)
    AtomicIsize(isize)
}

unsafe impl Atomic for AtomicBool {
    const INIT: Self = AtomicBool::new(false);
    type Value = bool;

    fn next(value: Self::Value) -> Option<Self::Value> {
        if value {
            None
        } else {
            Some(true)
        }
    }
}

unsafe impl Scalar for () {
    type Atomic = AtomicBool;
    type Error = ();

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        if value {
            Err(())
        } else {
            Ok(())
        }
    }
}

unsafe impl Scalar for [u8; 2] {
    type Atomic = AtomicU16;
    type Error = core::convert::Infallible;

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        Ok(value.to_ne_bytes())
    }
}

unsafe impl Scalar for [u8; 3] {
    type Atomic = AtomicU32;
    type Error = ();

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        match value.to_le_bytes() {
            [a, b, c, 0] => Ok([a, b, c]),
            _ => Err(()),
        }
    }
}

unsafe impl Scalar for [u8; 4] {
    type Atomic = AtomicU32;
    type Error = core::convert::Infallible;

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        Ok(value.to_ne_bytes())
    }
}

unsafe impl Scalar for [u8; 5] {
    type Atomic = AtomicU64;
    type Error = ();

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        match value.to_le_bytes() {
            [a, b, c, d, e, 0, 0, 0] => Ok([a, b, c, d, e]),
            _ => Err(()),
        }
    }
}

unsafe impl Scalar for [u8; 6] {
    type Atomic = AtomicU64;
    type Error = ();

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        match value.to_le_bytes() {
            [a, b, c, d, e, f, 0, 0] => Ok([a, b, c, d, e, f]),
            _ => Err(()),
        }
    }
}

unsafe impl Scalar for [u8; 7] {
    type Atomic = AtomicU64;
    type Error = ();

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        match value.to_le_bytes() {
            [a, b, c, d, e, f, g, 0] => Ok([a, b, c, d, e, f, g]),
            _ => Err(()),
        }
    }
}

unsafe impl Scalar for [u8; 8] {
    type Atomic = AtomicU64;
    type Error = core::convert::Infallible;

    fn try_from_atomic_inner(value: <Self::Atomic as Atomic>::Value) -> Result<Self, Self::Error> {
        Ok(value.to_ne_bytes())
    }
}
