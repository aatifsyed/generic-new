#![allow(dead_code)]
use generic_new::GenericNew;
use std::path::PathBuf;

#[derive(GenericNew)]
struct Foo {
    bar: PathBuf,
}

#[derive(GenericNew)]
struct FooTup(PathBuf);

fn main() {
    Foo::new("hello");
    FooTup::new("hello");
}
