use icell::scoped::{self, ICell};

fn main() {
    scoped::with(|foo| {
        let value = ICell::new(10);

        scoped::with(|bar| {
            foo.read(&value);
            bar.read(&value);
        });
    });
}
