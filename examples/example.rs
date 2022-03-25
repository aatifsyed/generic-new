use generic_new::GenericNew;
use derive_more::Deref;

#[derive(GenericNew, Deref)]
struct Foo {
    #[deref]
    #[generic_new(input_type = str, transform = |id| id )]
    food: Vec<usize>,
}

#[derive(GenericNew)]
struct Bar(usize, usize);

fn main() {}
