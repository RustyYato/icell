use icell::{
    scoped::{self, ICell},
    write_all,
};

#[test]
fn create() {
    scoped::owner!(owner);
    assert_eq!(std::mem::size_of_val(&owner), 0);
}

#[test]
fn read() {
    scoped::owner!(owner);
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let cell = ICell::<u32>::new(0xdead_beef);

    assert_eq!(*owner.read(&cell), 0xdead_beef);
}

#[test]
fn write() {
    scoped::owner!(owner);
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let cell = ICell::<u32>::new(0xdead_beef);

    let value = owner.write(&cell);
    *value = 0;

    assert_eq!(*owner.read(&cell), 0);
}

#[test]
fn write_all() {
    scoped::owner!(owner);
    assert_eq!(std::mem::size_of_val(&owner), 0);

    let a = ICell::<u32>::new(0xdead_beef);
    let b = ICell::<u32>::new(0xbeef_dead);
    let c = ICell::<u32>::new(0xdead_baaf);
    let d = ICell::<u32>::new(0xdeed_beef);

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
