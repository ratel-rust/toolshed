# Toolshed

This crate contains an `Arena` allocator, along with a few common data
structures that can be used in tandem with it.

For all those times when you need to create a recursively nested tree
of `enum`s and find yourself in pain having to put everything in
`Box`es all the time.

## Features

+ Paginated `Arena`: internally preallocates 64KiB _pages_ on the heap and
    allows `Copy` types to be put on that heap.

+ `CopyCell`: virtually identical to `std::cell::Cell` but requires that
    internal types implement `Copy`, and implements `Copy` itself.

+ `List`, `Map` and `Set`: your basic data structures that allocate on the
    `Arena` and use internal mutability via `CopyCell`. Never worry about
    sharing pointers again!

+ `BloomMap` and `BloomSet`: special variants of `Map` and `Set` with a
    very simple but very fast bloom filter. If a map / set is often queried
    for keys / elements it doesn't contain, the bloom filter check will
    reduce the need to do a full tree lookup, greatly increasing performance.
    The overhead compared to a regular `Map` or `Set` is also minimal.

+ All data structures implement expected traits, such as `Debug` or `PartialEq`.

+ Optional **serde** `Serialize` support behind a feature flag.

## Example

```rust
extern crate toolshed;

use toolshed::Arena;
use toolshed::map::Map;

// Only `Copy` types can be allocated on the `Arena`!
#[derive(Debug, PartialEq, Clone, Copy)]
enum Foo<'arena> {
    Integer(u64),

    // Recursive enum without `Box`es!
    Nested(&'arena Foo<'arena>),
}

fn main() {
    // Create a new arena
    let arena = Arena::new();

    // We allocate first instance of `Foo` in the arena.
    //
    // Please note that the `alloc` method returns a `&mut` reference.
    // Since we want to share our references around, we are going to
    // dereference and re-reference them to immutable ones with `&*`.
    let child: &Foo = &*arena.alloc(Foo::Integer(42));

    // Next instance of `Foo` will contain the child reference.
    let parent: &Foo = &*arena.alloc(Foo::Nested(child));

    // Empty map does not allocate
    let map = Map::new();

    // Inserting stuff in the map requires a reference to the `Arena`.
    // The reference can be shared, since `Arena` uses interior mutability.
    map.insert(&arena, "child", child);

    // We can put our `map` on the arena as well. Once again we use the `&*`
    // operation to change the reference to be immutable, just to demonstrate
    // that our `Map` implementation is perfectly happy with internal mutability.
    let map: &Map<&str, &Foo> = &*arena.alloc(map);

    // Each insert allocates a small chunk of data on the arena. Since arena is
    // preallocated on the heap, these inserts are very, very fast.
    //
    // We only have a non-mutable reference to `map` now, however `Map` is also
    // using interior mutability on references to allow exactly this kind of
    // behavior in a safe manner.
    map.insert(&arena, "parent", parent);

    assert_eq!(map.get("child"), Some(&Foo::Integer(42)));
    assert_eq!(map.get("parent"), Some(&Foo::Nested(&Foo::Integer(42))));
    assert_eq!(map.get("heh"), None);
}
```

## Benches

Here is a very biased benchmark of the different sets:

```
running 8 tests
test bloom_set_create  ... bench:          49 ns/iter (+/- 0)
test bloom_set_read    ... bench:         181 ns/iter (+/- 10)
test fxhash_set_create ... bench:          86 ns/iter (+/- 1)
test fxhash_set_read   ... bench:         312 ns/iter (+/- 4)
test hash_set_create   ... bench:         152 ns/iter (+/- 94)
test hash_set_read     ... bench:       1,105 ns/iter (+/- 1)
test set_create        ... bench:          37 ns/iter (+/- 0)
test set_read          ... bench:         440 ns/iter (+/- 1)
```

* `set` and `bloom_set` are benchmarks of `Set` and `BloomSet` from this crate.
* `hash_set` is the default stdlib `HashSet`.
* `fxhash_set` is a `HashSet` using the `fxhash` crate hash.

## License

This crate is distributed under the terms of both the MIT license
and the Apache License (Version 2.0). Choose whichever one works best for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
