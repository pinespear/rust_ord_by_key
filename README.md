# ord_by_key
Provides a convenient macro for implementing Ord trait with logic
specified in an inline expression

```rust
use core::cmp::Reverse;
// `Person` will be ordered by `last_name`, then by 
// `first_name`, then by `age` in reverse
#[ord_eq_by_key_selector(|p|
    &p.last_name,
    &p.first_name,
    Reverse(p.age),)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub age: usize,
}
```

```rust
// Container for `&str` which will be ordered by underlying
// string length
#[ord_eq_by_key_selector(|(s)| s.len())]
pub struct StrByLen<'a>(&'a str);

assert!(StrByLen("Alex") > StrByLen("Bob"));
```

## [`no_std`](https://rust-embedded.github.io/book/intro/no-std.html) support
`ord_by_key` should be compatible with `no_std`, but it was not tested.

## TODO

- [x] Better parameters syntax for structs with unnamed fields
- [ ] Support enums
- [ ] Support `_` in parameter definition
- [ ] Test with `no_std`

# License
Distributed under the terms of both the MIT license and the Apache License (Version 2.0)