use std::cmp::{max, min};
use std::collections::HashMap;
use unicode_normalization_source::{properties::*, UNICODE};

/// в каких границах находятся нестартеры?
/// сколько стартеров, сколько нестартеров?
fn main()
{
    let mut from = u32::MAX;
    let mut to = 0;

    let mut starters = 0;
    let mut nonstarters = 0;

    let unicode: &HashMap<u32, Codepoint> = &UNICODE;

    for codepoint in unicode.values() {
        if codepoint.ccc.is_starter() {
            starters += 1;
            continue;
        }

        nonstarters += 1;

        from = min(from, codepoint.code);
        to = max(to, codepoint.code);
    }

    println!(
        "\nнестартеры находятся в пределах диапазона: U+{:04X} ..= U+{:04X}\n",
        from, to
    );

    println!("стартеров (записанных в UnicodeData.txt): {}, нестартеров: {}\n", starters, nonstarters);
}

/*

результат:

нестартеры находятся в пределах диапазона: U+0300 ..= U+1E94A

стартеров: 148329, нестартеров: 922

*/
