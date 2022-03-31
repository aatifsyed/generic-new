#![allow(dead_code)]
use generic_new::GenericNew;
use derive_more::Deref; // Does another derive macro interfere?

#[derive(GenericNew, Deref)]
struct Foo {
    #[deref]
    bar: usize
}

#[derive(GenericNew, Deref)]
struct FooTup(usize);

fn main() {
    Foo::new(1usize);
    FooTup::new(1usize);
}