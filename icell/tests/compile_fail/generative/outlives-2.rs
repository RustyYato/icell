use icell::generative::{self as gen, ICell};

fn main() {
    gen::new!(owner);
    let value = ICell::new(10);

    let x = owner.read(&value);

    drop(value);

    assert_eq!(*x, 10);
}
