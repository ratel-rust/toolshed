#![feature(test)]
extern crate test;
extern crate fxhash;
extern crate toolshed;

use toolshed::set::{BloomSet, Set};
use toolshed::arena::Arena;
use test::{Bencher, black_box};
use fxhash::FxHashSet;

static WORDS: &[&str] = &["ARENA_BLOCK", "Arena", "Cell", "Self", "String", "T", "Vec", "_unchecked", "a", "alignment", "alloc", "alloc_bytes", "alloc_str", "alloc_str_zero_end", "alloc_string", "as", "as_bytes", "as_mut_ptr", "as_ptr", "block", "cap", "cell", "const", "copy_nonoverlapping", "else", "extend_from_slice", "fn", "from_raw_parts", "from_utf", "get", "grow", "if", "impl", "inline", "into", "into_bytes", "isize", "len", "len_with_zero", "let", "mem", "mut", "new", "offset", "ptr", "pub", "push", "replace", "return", "self", "set", "size_of", "slice", "std", "store", "str", "struct", "temp", "u", "unsafe", "use", "usize", "val", "vec", "with_capacity"];
static SET_WORDS: &[&str] = &["alloc_bytes", "alloc", "Cell", "String", "yetAnother"];

#[bench]
fn set_read(b: &mut Bencher) {
    let arena = Arena::new();
    let a     = &arena;
    let set = Set::new();

    for word in SET_WORDS.iter() {
        set.insert(a, *word);
    }

    b.iter(|| {
        for word in WORDS.iter() {
            black_box(set.contains(word));
        }
    })
}

#[bench]
fn set_create(b: &mut Bencher) {
    let arena = Arena::new();
    let a     = &arena;

    b.iter(|| {
        unsafe { a.clear() };
        let set = Set::new();

        for word in SET_WORDS.iter() {
            set.insert(a, *word);
        }

        black_box(set)
    })
}

#[bench]
fn bloom_set_read(b: &mut Bencher) {
    let arena = Arena::new();
    let a     = &arena;
    let set = BloomSet::new();

    for word in SET_WORDS.iter() {
        set.insert(a, *word);
    }

    b.iter(|| {
        for word in WORDS.iter() {
            black_box(set.contains(word));
        }
    })
}

#[bench]
fn bloom_set_create(b: &mut Bencher) {
    let arena = Arena::new();
    let a     = &arena;

    b.iter(|| {
        unsafe { a.clear() };
        let set = BloomSet::new();

        for word in SET_WORDS.iter() {
            set.insert(a, *word);
        }

        black_box(set)
    })
}

#[bench]
fn fxhash_set_read(b: &mut Bencher) {
    let mut set = FxHashSet::default();

    for word in SET_WORDS.iter() {
        set.insert(*word);
    }

    b.iter(|| {
        for word in WORDS.iter() {
            black_box(set.contains(word));
        }
    })
}

#[bench]
fn fxhash_set_create(b: &mut Bencher) {
    b.iter(|| {
        let mut set = FxHashSet::default();

        for word in SET_WORDS.iter() {
            set.insert(*word);
        }

        black_box(set)
    })
}
