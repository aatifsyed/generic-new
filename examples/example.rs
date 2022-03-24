use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    food: Vec<usize>,
}

#[derive(GenericNew)]
struct Bar(usize, usize);

fn main() {}
