#![feature(test)]
extern crate test;

use toolshed::list::ListBuilder;
use toolshed::Arena;
use test::{Bencher, black_box};

static WORDS: &[&str] = &[
    "ARENA_BLOCK", "Arena", "Cell", "Self", "String", "T", "Vec", "_unchecked", "a",
    "alignment", "alloc", "alloc_bytes", "alloc_str", "alloc_str_zero_end", "alloc_string",
    "as", "as_bytes", "as_mut_ptr", "as_ptr", "block", "cap", "cell", "const",
    "copy_nonoverlapping", "else", "extend_from_slice", "fn", "from_raw_parts", "from_utf",
    "get", "grow", "if", "impl", "inline", "into", "into_bytes", "isize", "len",
    "len_with_zero", "let", "mem", "mut", "new", "offset", "ptr", "pub", "push",
    "replace", "return", "self", "set", "size_of", "slice", "std", "store", "str",
    "struct", "temp", "u", "unsafe", "use", "usize", "val", "vec", "with_capacity"
];

#[bench]
fn vec_create_016(b: &mut Bencher) {
    let words = &WORDS[..16];

    b.iter(|| {
        let mut vec = Vec::new();

        for word in words.iter() {
            vec.push(word);
        }

        black_box(vec);
    })
}

#[bench]
fn vec_create_032(b: &mut Bencher) {
    let words = &WORDS[..32];

    b.iter(|| {
        let mut vec = Vec::new();

        for word in words.iter() {
            vec.push(word);
        }

        black_box(vec);
    })
}

#[bench]
fn vec_create_064(b: &mut Bencher) {
    let words = &WORDS[..64];

    b.iter(|| {
        let mut vec = Vec::new();

        for word in words.iter() {
            vec.push(word);
        }

        black_box(vec);
    })
}

#[bench]
fn vec_create_256(b: &mut Bencher) {
    b.iter(|| {
        let mut vec = Vec::new();

        for i in 0..256usize {
            vec.push((i, i));
        }

        black_box(vec);
    })
}

#[bench]
fn list_create_016(b: &mut Bencher) {
    let arena = Arena::new();
    let words = &WORDS[1..16];

    b.iter(|| {
        unsafe { arena.clear() };
        let builder = ListBuilder::new(&arena, WORDS[0]);

        for word in words.iter() {
            builder.push(&arena, *word);
        }

        black_box(builder.as_list());
    })
}

#[bench]
fn list_create_032(b: &mut Bencher) {
    let arena = Arena::new();
    let words = &WORDS[1..32];

    b.iter(|| {
        unsafe { arena.clear() };
        let builder = ListBuilder::new(&arena, WORDS[0]);

        for word in words.iter() {
            builder.push(&arena, *word);
        }

        black_box(builder.as_list());
    })
}

#[bench]
fn list_create_064(b: &mut Bencher) {
    let arena = Arena::new();
    let words = &WORDS[1..64];

    b.iter(|| {
        unsafe { arena.clear() };
        let builder = ListBuilder::new(&arena, WORDS[0]);

        for word in words.iter() {
            builder.push(&arena, *word);
        }

        black_box(builder.as_list());
    })
}

#[bench]
fn list_create_256(b: &mut Bencher) {
    let arena = Arena::new();

    b.iter(|| {
        unsafe { arena.clear() };
        let builder = ListBuilder::new(&arena, (0usize, 0));

        for i in 1..256usize {
            builder.push(&arena, (i, i));
        }

        black_box(builder.as_list());
    })
}
