#![cfg_attr(not(feature = "std"), no_std)]

#[macro_export]
macro_rules! write_all {
    ($owner:expr => $($cells:expr),*) => {{
        $crate::write_all!(@destruct [()] [$(($cells))*] $owner.write_all($crate::hlist!($(&$cells as &$crate::ICell<_, _>),*)))
    }};
    (@destruct [$($rest:tt)*] [$first:tt $($cells:tt)*] $value:expr) => {
        match $value {
            $crate::hlist::Cons(value, rest) => $crate::write_all!(@destruct [$($rest)* , value] [$($cells)*] rest)
        }
    };
    (@destruct [()] [] $value:expr) => {
        match $value { $crate::hlist::Nil => () }
    };
    (@destruct [(), $($rest:tt)*] [] $value:expr) => {
        match $value { $crate::hlist::Nil => ($($rest)*) }
    };
}

mod core;
pub use self::core::{ICell, Identifier, Owner, Transparent};

#[doc(hidden)]
pub mod hlist;

pub mod generative;
pub mod immovable;
pub mod runtime;
pub mod typeid;
#[cfg(feature = "std")]
pub mod typeid_tl;
