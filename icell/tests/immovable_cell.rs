use icell::{immovable::Immovable, write_all};

#[test]
fn create() {
    let owner = Immovable::owner();
    assert_eq!(std::mem::size_of_val(&owner), 1);
}

#[test]
fn read() {
    let owner = Immovable::owner();
    assert_eq!(std::mem::size_of_val(&owner), 1);

    let cell = owner.cell::<u32>(0xdead_beef);

    assert_eq!(*owner.read(&cell), 0xdead_beef);
}

#[test]
fn write() {
    let mut owner = Immovable::owner();
    assert_eq!(std::mem::size_of_val(&owner), 1);

    let cell = owner.cell::<u32>(0xdead_beef);

    let value = owner.write(&cell);
    *value = 0;

    assert_eq!(*owner.read(&cell), 0);
}

#[test]
fn write_all() {
    let mut owner = Immovable::owner();
    assert_eq!(std::mem::size_of_val(&owner), 1);

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
    let owner = &mut Immovable::owner();
    assert_eq!(std::mem::size_of_val(owner), 1);

    let cell = Immovable::owner().cell::<u32>(0xdead_beef);

    Immovable::owner().read(&cell);
}
