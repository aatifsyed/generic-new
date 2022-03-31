# generic-new

<div align="center">

[![crates-io](https://img.shields.io/crates/v/generic-new.svg)](https://crates.io/crates/generic-new)
[![docs-rs](https://docs.rs/generic-new/badge.svg)](https://docs.rs/generic-new)
[![github](https://img.shields.io/static/v1?label=&message=github&color=grey&logo=github)](https://github.com/aatifsyed/generic-new)

</div>

The `GenericNew` derive macro will generate an ergonomic constructor which contains shortcuts for certain types.

```rust
use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    s: String,     // -> impl AsRef<str>
    v: Vec<usize>, // -> impl IntoIterator<Item = usize>
    p: PathBuf,    // -> impl AsRef<Path>
    #[generic_new(ignore)]
    i: String,     // Turn off magic conversion for some fields
    #[generic_new(ty = impl Into<usize>, converter = |u|Into::into(u))]
    u: usize,      // Custom converters are supported
}

Foo::new(
    "hello",
    [1, 2, 3],
    "path/to/foo",
    String::from("world"),
    1u16,
);

```
