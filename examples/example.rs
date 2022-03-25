use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    #[generic_new("foo")]
    food: Vec<usize>,
}

#[derive(GenericNew)]
struct Bar(usize, usize);

fn main() {}
