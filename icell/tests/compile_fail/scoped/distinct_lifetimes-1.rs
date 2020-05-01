use icell::scoped::{self, ICell};

fn main() {
    scoped::owner!(foo);
    scoped::owner!(bar);
    let value = ICell::new(10);

    foo.read(&value);
    bar.read(&value);
}
