#![allow(dead_code)]
use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    bar: Vec<usize>,
}

#[derive(GenericNew)]
struct FooTup(Vec<usize>);

fn main() {
    Foo::new([1usize, 2usize]);
    FooTup::new([1, 2, 3]);
}
