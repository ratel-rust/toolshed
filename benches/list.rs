#![feature(test)]
extern crate test;
extern crate fxhash;
extern crate toolshed;

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
fn vec_create(b: &mut Bencher) {
    b.iter(|| {
        let mut vec = Vec::new();

        for word in WORDS.iter() {
            vec.push(word);
        }

        black_box(vec);
    })
}

#[bench]
fn list_create(b: &mut Bencher) {
    use toolshed::new_list::ListBuilder;

    let arena = Arena::new();
    let a     = &arena;

    b.iter(|| {
        unsafe { a.clear() };
        let mut builder = ListBuilder::new(a);

        for word in WORDS.iter() {
            builder.push(*word);
        }

        black_box(builder.into_list());
    })
}
