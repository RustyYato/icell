use icell::runtime_id;

runtime_id!(type Runtime(usize));

fn main() {
    let owner = Runtime::owner();
    let value = owner.cell(10);

    let x = owner.read(&value);

    drop(owner);

    assert_eq!(*x, 10);
}
