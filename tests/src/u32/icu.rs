use icu_normalizer::DecomposingNormalizer as icu;
use unicode_decomposing_u32 as u32_codepoints;

/// сравниваем с результатами нормализации ICU
#[test]
fn icu()
{
    let icu_nfd = icu::new_nfd();
    let icu_nfkd = icu::new_nfkd();

    let nfd = u32_codepoints::new_nfd();
    let nfkd = u32_codepoints::new_nfkd();

    for data in crate::data::files() {
        let result: String = nfd
            .normalize(data.1.as_str())
            .iter()
            .map(|c| char::from_u32(c.code()).unwrap())
            .collect();

        assert_eq!(
            result,
            icu_nfd.normalize(data.1.as_str()),
            "nfd u32 - {}",
            data.0
        );

        let result: String = nfkd
            .normalize(data.1.as_str())
            .iter()
            .map(|c| char::from_u32(c.code()).unwrap())
            .collect();

        assert_eq!(
            result,
            icu_nfkd.normalize(data.1.as_str()),
            "nfkd u32 - {}",
            data.0
        );
    }
}
