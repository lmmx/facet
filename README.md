
# facet

[![experimental](https://img.shields.io/badge/status-experimental-yellow)](https://github.com/fasterthanlime/facet)
[![free of syn](https://img.shields.io/badge/free%20of-syn-hotpink)](https://github.com/fasterthanlime/free-of-syn)
[![crates.io](https://img.shields.io/crates/v/facet.svg)](https://crates.io/crates/facet)
[![documentation](https://docs.rs/facet/badge.svg)](https://docs.rs/facet)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/facet.svg)](./LICENSE)

Thanks to all individual and corporate sponsors, without whom this work could not exist:

<p> <a href="https://ko-fi.com/fasterthanlime">
    <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/ko-fi-dark.svg">
    <img src="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/ko-fi-light.svg" height="40" alt="Ko-fi">
    </picture>
</a> <a href="https://github.com/sponsors/fasterthanlime">
    <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/github-dark.svg">
    <img src="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/github-light.svg" height="40" alt="GitHub Sponsors">
    </picture>
</a> <a href="https://patreon.com/fasterthanlime">
    <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/patreon-dark.svg">
    <img src="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/patreon-light.svg" height="40" alt="Patreon">
    </picture>
</a> &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; <a href="https://zed.dev">
    <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/zed-dark.svg">
    <img src="https://github.com/facet-rs/facet/raw/main/static/sponsors-v2/zed-light.svg" height="40" alt="Zed">
    </picture>
</a> </p>
             

facet provides type reflection for Rust, in const contexts.

Any type that implements `Facet` trait returns a `Shape`, which describes:

  * The memory layout of the type
  * Its definition: struct fields, underlying type for newtypes, etc.
  * How to drop it in place

The `Poke` type is able to allocate (or work from a `&mut MaybeUninit<T>`)
any Facet type, and gradually initialize its fields — until the fully-built
value is moved out of the partial.

The `Peek` type helps perform read operations on any Facet type.

It comes with a derive macro that uses [unsynn](https://crates.io/crates/unsynn)
for speed of compilation.

## Ecosystem

There are separate crates for the trait, the core types, the derive macro, peek, poke,
and various serializers, deserializers, and pretty printers etc.

The hub for everything is the [facet](https://crates.io/crates/facet) crate.

You can start with its [README](https://github.com/facet-rs/facet/blob/main/facet/README.md).


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/facet-rs/facet/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/facet-rs/facet/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.