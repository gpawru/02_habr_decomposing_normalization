use icu_normalizer::DecomposingNormalizer as icu;
use unicode_decomposing::DecomposingNormalizer as my;

/// сравниваем с результатами нормализации ICU
#[test]
fn icu()
{
    let icu_nfd = icu::new_nfd();
    let icu_nfkd = icu::new_nfkd();

    macro_rules! test {
        ($(($nfd: expr, $nfkd: expr,  $t: expr)),+) => {
            $(
                let nfd = $nfd;
                let nfkd = $nfkd;

                for data in crate::data::files() {
                    assert_eq!(
                        nfd.normalize(data.1.as_str()),
                        icu_nfd.normalize(data.1.as_str()),
                        "nfd,  {} - {}",
                        $t,
                        data.0
                    );
                    assert_eq!(
                        nfkd.normalize(data.1.as_str()),
                        icu_nfkd.normalize(data.1.as_str()),
                        "nfkd, {} - {}",
                        $t,
                        data.0
                    );
                }
            )+
        };
    }

    test!((my::new_nfd(), my::new_nfkd(), "my"));
}
