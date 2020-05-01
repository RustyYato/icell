use icell::immovable::Immovable;

fn main() {
    let owner = Immovable::owner();
    let value = owner.cell(10);

    let x = owner.read(&value);

    drop(owner);

    assert_eq!(*x, 10);
}
