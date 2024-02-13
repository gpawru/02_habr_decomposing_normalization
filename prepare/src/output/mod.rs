use std::fs::File;
use std::io::Write;

use self::stats::format_codepoint_groups;
use crate::format_num_vec;
use crate::tables::STARTING_CODEPOINTS_BLOCK;

pub mod format;
pub mod stats;

/// длина строки в файле с подготовленными данными
const FORMAT_STRING_LENGTH: usize = 120;

/// пишем данные о декомпозиции
pub fn write(canonical: bool, file: &mut File, stats_file: &mut File)
{
    let (index, data, expansions, stats) = crate::tables::prepare(canonical);

    let name = match canonical {
        true => "NFD",
        false => "NFKD",
    };

    let output = format!(
        "DecompositionData {{\n  \
            index: &[{}  ],\n  \
            data: &[{}  ],\n  \
            expansions: &[{}  ],\n  \
            continuous_block_end: 0x{:04X},\n\
        }}\n",
        format_num_vec(index.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(data.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(expansions.as_slice(), FORMAT_STRING_LENGTH),
        STARTING_CODEPOINTS_BLOCK,
    );

    write!(file, "{}", output).unwrap();
    write!(stats_file, "{}", format_codepoint_groups(stats)).unwrap();

    stats::print(
        name,
        index.as_slice(),
        data.as_slice(),
        expansions.as_slice(),
    );
}
