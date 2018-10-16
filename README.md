# metatype

[![Crates.io](https://img.shields.io/crates/v/metatype.svg?style=flat-square&maxAge=86400)](https://crates.io/crates/metatype)
[![Apache-2.0 licensed](https://img.shields.io/crates/l/metatype.svg?style=flat-square&maxAge=2592000)](LICENSE.txt)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/alecmocatta/metatype?branch=master&svg=true)](https://ci.appveyor.com/project/alecmocatta/metatype)
[![Build Status](https://circleci.com/gh/alecmocatta/metatype/tree/master.svg?style=shield)](https://circleci.com/gh/alecmocatta/metatype)
[![Build Status](https://travis-ci.com/alecmocatta/metatype.svg?branch=master)](https://travis-ci.com/alecmocatta/metatype)

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
Licensed under Apache License, Version 2.0, ([LICENSE.txt](LICENSE.txt) or http://www.apache.org/licenses/LICENSE-2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
