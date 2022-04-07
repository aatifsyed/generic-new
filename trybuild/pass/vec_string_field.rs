#![allow(dead_code)]
use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    bar: Vec<String>,
}

#[derive(GenericNew)]
struct FooTup(Vec<String>);

fn main() {
    Foo::new(["hello", "world"]);
    FooTup::new(["a", "b", "c"]);
}
