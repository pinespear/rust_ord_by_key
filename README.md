# rust_ord_by_key
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
#[ord_eq_by_key_selector(|s| s.0.len())]
pub struct StrByLen<'a> (&'a str);

assert!(StrByLen("Alex") > StrByLen("Bob"));
```

# License
Distributed under the terms of both the MIT license and the Apache License (Version 2.0)