
# facet-types

[![experimental](https://img.shields.io/badge/status-experimental-yellow)](https://github.com/fasterthanlime/facet)
[![free of syn](https://img.shields.io/badge/free%20of-syn-hotpink)](https://github.com/fasterthanlime/free-of-syn)
[![crates.io](https://img.shields.io/crates/v/facet-types.svg)](https://crates.io/crates/facet-types)
[![documentation](https://docs.rs/facet-types/badge.svg)](https://docs.rs/facet-types)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/facet-types.svg)](./LICENSE)

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
             

Defines the core types used throughout the facet ecosystem for runtime reflection:

* `Shape`: The central type that describes the memory layout and capabilities of a type
* Various vtables that define how to manipulate types at runtime
* The `Def` tree, which describes type definitions (structs, enums, etc.)

This crate is foundational to facet's reflection capabilities, providing the type system that enables safe runtime type manipulation.

### Key Components

* Memory layout information (size, alignment, etc.)
* Type definition hierarchies
* Runtime type manipulation vtables
* Safe type erasure primitives


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/facet-rs/facet/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/facet-rs/facet/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.