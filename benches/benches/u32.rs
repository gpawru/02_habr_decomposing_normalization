use criterion::{criterion_group, criterion_main, Criterion};
use unicode_decomposing_u32::new_nfd;
use unicode_decomposing_u32::new_nfkd;
use unicode_decomposing_u32::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "u32",
    new_nfd(),
    DecomposingNormalizer
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "u32",
    new_nfkd(),
    DecomposingNormalizer
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "u32",
    new_nfd(),
    DecomposingNormalizer
);

criterion_group!(benches, nfd, nfkd, dec);
criterion_main!(benches);
