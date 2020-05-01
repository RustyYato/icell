use icell::scoped::{self, ICell};

fn main() {
    scoped::owner!(owner);
    let value = ICell::new(10);

    let x = owner.read(&value);

    drop(value);

    assert_eq!(*x, 10);
}
