use icell::generative::{self as gen, ICell};

fn main() {
    gen::new!(foo);
    gen::new!(bar);
    let value = ICell::new(10);

    foo.read(&value);
    bar.read(&value);
}
