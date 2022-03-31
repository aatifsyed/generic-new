#![allow(dead_code)]
use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    bar: String,
}

#[derive(GenericNew)]
struct FooTup(String);

fn main() {
    Foo::new("hello");
    FooTup::new("hello");
}
