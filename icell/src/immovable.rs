use std::mem::MaybeUninit;

pub type Owner<Im = MaybeUninit<u8>> = crate::Owner<Immovable<Im>>;
pub type ICell<T> = crate::ICell<Address, T>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Immovable<T = MaybeUninit<u8>>(pub T, ());

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(usize);

impl Immovable {
    pub const fn new() -> Self {
        Self::with_value(MaybeUninit::uninit())
    }

    pub fn owner() -> Owner {
        Owner::new(Self::new())
    }
}

impl<Im> Immovable<Im> {
    #[allow(clippy::unnecessary_operation)]
    pub const fn with_value(value: Im) -> Self {
        // assert that the size is non-zero
        // any value with a non-zero size will have a unique address
        [()][(std::mem::size_of::<Im>() == 0) as usize];
        unsafe { Self::with_value_unchecked(value) }
    }

    /// # Safety
    ///
    /// the given value must have a unique address (this may not be the case for
    /// zero-sized types)
    pub const unsafe fn with_value_unchecked(value: Im) -> Self {
        Self(value, ())
    }

    pub fn owner_with_value(value: Im) -> Owner<Im> {
        Owner::new(Self::with_value(value))
    }

    pub fn cell<T>(&self, value: T) -> ICell<T> {
        use crate::Identifier;
        self.id().cell(value)
    }
}

impl Address {
    pub fn cell<T>(self, value: T) -> ICell<T> {
        unsafe { ICell::from_raw_parts(self, value) }
    }
}

unsafe impl<Im> crate::Identifier for Immovable<Im> {
    type Id = Address;

    fn id(&self) -> Self::Id {
        Address(self as *const Immovable<Im> as usize)
    }

    fn check_id(&self, &id: &Self::Id) -> bool {
        self.id() == id
    }
}
