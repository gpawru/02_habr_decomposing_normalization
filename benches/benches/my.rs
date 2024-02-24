use criterion::{criterion_group, criterion_main, Criterion};
use unicode_decomposing::nfd_normalizer;
use unicode_decomposing::nfkd_normalizer;
use unicode_decomposing::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "my",
    nfd_normalizer(),
    DecomposingNormalizer
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "my",
    nfkd_normalizer(),
    DecomposingNormalizer
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "my",
    nfd_normalizer(),
    DecomposingNormalizer
);

criterion_group!(benches, nfd, nfkd, dec);
criterion_main!(benches);
