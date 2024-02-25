use unicode_data::{NormalizationTest, NORMALIZATION_TESTS};
use unicode_decomposing as optimized;
use unicode_decomposing_basic as basic;

macro_rules! test {
    ($left: expr, $right: expr, $normalizer: expr, $test: expr, $str: expr) => {
        assert_eq!(
            $left,
            $normalizer.normalize(&$right),
            stringify!($str),
            $test.line + 1,
            $test.description
        );
    };
}

/// тесты NFD нормализации из UCD
#[test]
fn ucd_test_nfd()
{
    // c3 ==  toNFD(c1) ==  toNFD(c2) ==  toNFD(c3)
    // c5 ==  toNFD(c4) ==  toNFD(c5)

    let tests: &Vec<NormalizationTest> = &NORMALIZATION_TESTS;

    macro_rules! test_group {
        ($($normalizer: expr),+) => {
            $(
                let normalizer = $normalizer;

                for t in tests {
                    test!(t.c3, t.c1, normalizer, t, "{} {}: c3 == toNFD(c1)");
                    test!(t.c3, t.c2, normalizer, t, "{} {}: c3 == toNFD(c2)");
                    test!(t.c3, t.c3, normalizer, t, "{} {}: c3 == toNFD(c3)");
                    test!(t.c5, t.c4, normalizer, t, "{} {}: c5 == toNFD(c4)");
                    test!(t.c5, t.c5, normalizer, t, "{} {}: c5 == toNFD(c5)");
                }
            )+
        };
    }

    test_group!(basic::new_nfd(), optimized::new_nfd());
}

/// тесты NFKD нормализации из UCD
#[test]
fn ucd_test_nfkd()
{
    // c5 == toNFKD(c1) == toNFKD(c2) == toNFKD(c3) == toNFKD(c4) == toNFKD(c5)

    let tests: &Vec<NormalizationTest> = &NORMALIZATION_TESTS;

    macro_rules! test_group {
        ($($normalizer: expr),+) => {
            $(
            let normalizer = $normalizer;

            for t in tests {
                test!(t.c5, t.c1, normalizer, t, "{} {}: c5 == toNFKD(c1)");
                test!(t.c5, t.c2, normalizer, t, "{} {}: c5 == toNFKD(c2)");
                test!(t.c5, t.c3, normalizer, t, "{} {}: c5 == toNFKD(c3)");
                test!(t.c5, t.c4, normalizer, t, "{} {}: c5 == toNFKD(c4)");
                test!(t.c5, t.c5, normalizer, t, "{} {}: c5 == toNFKD(c5)");
            }
        )+
        };
    }

    test_group!(basic::new_nfkd(), optimized::new_nfkd());
}
