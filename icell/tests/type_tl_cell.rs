#![cfg(feature = "std")]

use icell::{
    typeid_tl::{self, ICell},
    write_all,
};

#[test]
fn create() {
    typeid_tl::make!(type TestCreate);

    let owner = TestCreate::owner();
    assert_eq!(std::mem::size_of_val(&owner), 0);
}

#[test]
#[cfg(feature = "std")] // on `no_std` reentrant acquires block
#[cfg_attr(miri, ignore)]
#[should_panic = "attempted a reentrant acquire of a `Type<TestReentrant>`"]
fn reentrant() {
    typeid_tl::make!(type TestReentrant);

    let _owner = TestReentrant::owner();
    assert!(TestReentrant::try_owner().is_none());
    let _owner = TestReentrant::owner();
}

typeid_tl::make!(type Test);

#[test]
fn read() {
    let owner = Test::owner();
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let cell = ICell::new(0xdead_beef_u32);

    assert_eq!(*owner.read(&cell), 0xdead_beef);
}

#[test]
fn write() {
    let mut owner = Test::owner();
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let cell = ICell::new(0xdead_beef_u32);

    let value = owner.write(&cell);
    *value = 0;

    assert_eq!(*owner.read(&cell), 0);
}

#[test]
fn write_all() {
    let mut owner = Test::owner();
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let a = ICell::new(0xdead_beef_u32);
    let b = ICell::new(0xbeef_dead_u32);
    let c = ICell::new(0xdead_baaf_u32);
    let d = ICell::new(0xdeed_beef_u32);

    {
        let (a, b, c, d) = write_all!(owner => a, b, c, d);

        std::mem::swap(a, b);
        std::mem::swap(c, d);
        std::mem::swap(a, d);
    }

    let &a = owner.read(&a);
    let &b = owner.read(&b);
    let &c = owner.read(&c);
    let &d = owner.read(&d);

    assert_eq!(a, 0xdead_baaf);
    assert_eq!(b, 0xdead_beef);
    assert_eq!(c, 0xdeed_beef);
    assert_eq!(d, 0xbeef_dead);
}

#[test]
fn read_from_fresh() {
    let owner = Test::owner();
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let cell = owner.cell::<u32>(0xdead_beef);

    drop(owner);
    let owner = Test::owner();

    assert_eq!(*owner.read(&cell), 0xdead_beef);
}
