use criterion::{criterion_group, criterion_main, Criterion};
use unicode_decomposing::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "my",
    DecomposingNormalizer::nfd()
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "my",
    DecomposingNormalizer::nfkd()
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "my",
    DecomposingNormalizer::nfd()
);

criterion_group!(benches, nfd, nfkd, dec);
criterion_main!(benches);
