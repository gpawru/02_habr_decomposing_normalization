// use criterion::{criterion_group, criterion_main, Criterion};
// use unicode_decomposing::nfd_normalizer_u32;
// use unicode_decomposing::nfkd_normalizer_u32;
// use unicode_decomposing::DecomposingNormalizer;
// use unicode_decomposing::NFD;
// use unicode_decomposing::NFKD;
// use unicode_decomposing::Codepoint;

// mod group;

// group!(
//     "./../test_data/texts",
//     nfd,
//     test_nfd,
//     "nfd",
//     "u32",
//     nfd_normalizer_u32(),
//     DecomposingNormalizer<Vec<Codepoint>, NFD>
// );

// group!(
//     "./../test_data/texts",
//     nfkd,
//     test_nfkd,
//     "nfkd",
//     "u32",
//     nfkd_normalizer_u32(),
//     DecomposingNormalizer<Vec<Codepoint>, NFKD>
// );

// group!(
//     "./../test_data/texts_decomposed",
//     dec,
//     test_dec,
//     "dec",
//     "u32",
//     nfd_normalizer_u32(),
//     DecomposingNormalizer<Vec<Codepoint>, NFD>
// );

// criterion_group!(benches, nfd, nfkd, dec);
// criterion_main!(benches);
