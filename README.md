# Toolshed

This crate contains an `Arena` allocator, along with a few common data
structures that can be used in tandem with it.

For all those times when you need to create a recursively nested tree
of `enum`s and find yourself in pain having to put everything in
`Box`es all the time.

## Benches

Here is a very nonobjective test of the different sets as an example:

```
running 8 tests
test bloom_set_create  ... bench:          44 ns/iter (+/- 0)
test bloom_set_read    ... bench:         182 ns/iter (+/- 1)
test fxhash_set_create ... bench:          88 ns/iter (+/- 1)
test fxhash_set_read   ... bench:         312 ns/iter (+/- 26)
test hash_set_create   ... bench:         153 ns/iter (+/- 1)
test hash_set_read     ... bench:       1,123 ns/iter (+/- 2)
test set_create        ... bench:          33 ns/iter (+/- 0)
test set_read          ... bench:         442 ns/iter (+/- 2)
```

* `set` and `bloom_set` are benchmarks of `Set` and `BloomSet` from this crate.
* `hash_set` is the default stdlib `HashSet`.
* `fxhash_set` is a `HashSet` using the `fxhash` crate hash.

## License

This crate is distributed under the terms of both the MIT license
and the Apache License (Version 2.0). Choose whichever one works best for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
