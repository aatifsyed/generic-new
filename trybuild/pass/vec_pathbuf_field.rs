#![allow(dead_code)]
use generic_new::GenericNew;
use std::path::PathBuf;

#[derive(GenericNew)]
struct Foo {
    bar: Vec<PathBuf>,
}

#[derive(GenericNew)]
struct FooTup(Vec<PathBuf>);

fn main() {
    Foo::new(["/ab/solute", "./rel/ative"]);
    FooTup::new(["a", "b", "c"]);
}
