use icell::{runtime::Runtime, write_all};

#[test]
fn create() {
    let owner = Runtime::owner();
    assert_eq!(std::mem::size_of_val(&owner), 6);
}

#[test]
#[cfg(feature = "std")]
fn create_once_with_reuse() {
    icell::runtime_id!(type Once(()););
    icell::global_reuse!(type OnceReuse(Once));

    type Runtime = icell::runtime::Runtime<Once, OnceReuse>;

    let owner = Runtime::with_counter_and_reuse(OnceReuse);
    assert!(Runtime::try_with_counter_and_reuse(OnceReuse).is_err());
    assert_eq!(std::mem::size_of_val(&owner), 0);
    drop(owner);
    assert!(Runtime::try_with_counter_and_reuse(OnceReuse).is_ok());
}

#[test]
fn read() {
    let owner = Runtime::owner();
    assert_eq!(std::mem::size_of_val(&owner), 6);

    let cell = owner.cell::<u32>(0xdead_beef);

    assert_eq!(*owner.read(&cell), 0xdead_beef);
}

#[test]
fn write() {
    let mut owner = Runtime::owner();
    assert_eq!(std::mem::size_of_val(&owner), 6);

    let cell = owner.cell::<u32>(0xdead_beef);

    let value = owner.write(&cell);
    *value = 0;

    assert_eq!(*owner.read(&cell), 0);
}

#[test]
fn write_all() {
    let mut owner = Runtime::owner();
    assert_eq!(std::mem::size_of_val(&owner), 6);

    let a = owner.cell::<u32>(0xdead_beef);
    let b = owner.cell::<u32>(0xbeef_dead);
    let c = owner.cell::<u32>(0xdead_baaf);
    let d = owner.cell::<u32>(0xdeed_beef);

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
#[cfg_attr(miri, ignore)]
#[should_panic = "Tried to read using an unrelated owner"]
fn read_unrelated() {
    let owner = Runtime::owner();
    assert_eq!(std::mem::size_of_val(&owner), 6);

    let cell = owner.cell::<u32>(0xdead_beef);

    let owner = Runtime::owner();

    assert_eq!(*owner.read(&cell), 0xdead_beef);
}
