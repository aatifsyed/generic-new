#![allow(dead_code)]
use generic_new::GenericNew;

fn get_string_length(s: impl AsRef<str>) -> usize {
    s.as_ref().len()
}

#[derive(GenericNew)]
struct Foo {
    #[generic_new(ty = &str, converter = |s: &str| s.len())]
    bar: usize,
    #[generic_new(ty = impl AsRef<str>, converter = get_string_length)]
    baz: usize,
}

#[derive(GenericNew)]
struct FooTup(
    #[generic_new(ty = &str, converter = |s: &str| s.len())] usize,
    #[generic_new(ty = impl AsRef<str>, converter = get_string_length)] usize,
);

fn main() {
    Foo::new("hello", "world");
    FooTup::new("hello", "world");
}
