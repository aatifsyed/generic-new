# generic-new

<div align="center">

[![crates-io](https://img.shields.io/crates/v/generic-new.svg)](https://crates.io/crates/generic-new)
[![docs-rs](https://docs.rs/generic-new/badge.svg)](https://docs.rs/generic-new)
[![github](https://img.shields.io/static/v1?label=&message=github&color=grey&logo=github)](https://github.com/aatifsyed/generic-new)

</div>

A derive macro which generates an ergonomic constructor with shortcuts for certain types.

```rust
use generic_new::GenericNew;

#[derive(GenericNew)]
struct Foo {
    s: String,      // -> impl AsRef<str>
    v: Vec<usize>,  // -> impl IntoIterator<Item = usize>
    i: Vec<String>, // -> impl IntoIterator<Item = impl AsRef<str>>
    p: PathBuf,     // -> impl AsRef<Path>
    #[generic_new(ignore)]
    o: String,      // Turn off magic conversion for some fields
    #[generic_new(ty = impl Into<usize>, converter = |u|Into::into(u))]
    u: usize,       // Custom converters are supported
}

Foo::new(
    "hello",
    [1, 2, 3],
    ["a", "b", "c"],
    "path/to/foo",
    String::from("world"),
    1u16,
);

```

License: MIT
