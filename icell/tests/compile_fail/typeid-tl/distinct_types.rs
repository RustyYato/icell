use icell::typeid_tl::{make, ICell};

make!(type Foo);
make!(type Bar);

fn main() {
    let foo = Foo::owner();
    let bar = Bar::owner();
    let value = ICell::new(10);

    foo.read(&value);
    bar.read(&value);
}
