#![allow(dead_code)]
use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    #[generic_new]
    bar: usize
}

#[derive(GenericNew)]
struct FooTup(usize);

fn main() {
    Foo::new(1usize);
    FooTup::new(1usize);
}