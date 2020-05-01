use icell::typeid::{make, ICell};

make!(type Id);

fn main() {
    let owner = Id::owner();
    let value = ICell::new(10);

    let x = owner.read(&value);

    drop(value);

    assert_eq!(*x, 10);
}
