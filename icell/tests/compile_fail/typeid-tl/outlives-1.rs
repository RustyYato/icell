use icell::typeid_tl::{make, ICell};

make!(type Id);

fn main() {
    let owner = Id::owner();
    let value = ICell::new(10);

    let x = owner.read(&value);

    drop(owner);

    assert_eq!(*x, 10);
}
