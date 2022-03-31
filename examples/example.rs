use derive_more::Deref;
use generic_new::GenericNew;

#[derive(GenericNew, Deref)]
struct Foo {
    // #[deref]
    #[generic_new(ignore)]
    food: Vec<usize>,
}

#[derive(GenericNew)]
struct Bar(usize, usize);

fn main() {}
