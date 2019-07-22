# metatype

[![Crates.io](https://img.shields.io/crates/v/metatype.svg?maxAge=86400)](https://crates.io/crates/metatype)
[![MIT / Apache 2.0 licensed](https://img.shields.io/crates/l/metatype.svg?maxAge=2592000)](#License)
[![Build Status](https://dev.azure.com/alecmocatta/metatype/_apis/build/status/tests?branchName=master)](https://dev.azure.com/alecmocatta/metatype/_build/latest?branchName=master)

[Docs](https://docs.rs/metatype/0.1.1)

Helper methods to determine whether a type is `TraitObject`, `Slice` or `Concrete`, and work with them respectively.

## Examples

```rust
assert_eq!(usize::METATYPE, MetaType::Concrete);
assert_eq!(any::Any::METATYPE, MetaType::TraitObject);
assert_eq!(<[u8]>::METATYPE, MetaType::Slice);

let a: Box<usize> = Box::new(123);
assert_eq!((&*a).meta_type(), MetaType::Concrete);
let a: Box<any::Any> = a;
assert_eq!((&*a).meta_type(), MetaType::TraitObject);

let a = [123,456];
assert_eq!(a.meta_type(), MetaType::Concrete);
let a: &[i32] = &a;
assert_eq!(a.meta_type(), MetaType::Slice);

let a: Box<any::Any> = Box::new(123);
// https://github.com/rust-lang/rust/issues/50318
// let meta: TraitObject = (&*a).meta();
// println!("vtable: {:?}", meta.vtable);
```

## Note

This currently requires Rust nightly for the `raw` and `specialization` features.

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
