use criterion::{criterion_group, criterion_main, Criterion};
use unicode_decomposing::new_nfd;
use unicode_decomposing::new_nfkd;
use unicode_decomposing::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "my",
    new_nfd(),
    DecomposingNormalizer
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "my",
    new_nfkd(),
    DecomposingNormalizer
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "my",
    new_nfd(),
    DecomposingNormalizer
);

criterion_group!(benches, nfd, nfkd, dec);
criterion_main!(benches);
