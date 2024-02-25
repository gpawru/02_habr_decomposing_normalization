use unicode_data::UNICODE;

use crate::encode::{encode_codepoint, MARKER_HANGUL};
use crate::output::stats::CodepointGroups;

/// до этого кодпоинта (включительно) все кодпоинты записаны в таблицу данных последовательно
pub const STARTING_CODEPOINTS_BLOCK: u32 = 0xFFF;
/// последний кодпоинт таблицы с декомпозицией
pub const LAST_DECOMPOSITION_CODE: u32 = 0x2FA1D;
/// количество бит, с помощью которых может быть закодирован индекс блока
pub const BLOCK_BITS: u32 = 7;
/// максимально возможное количество блоков
pub const MAX_BLOCKS: u32 = LAST_DECOMPOSITION_CODE >> BLOCK_BITS;

#[macro_export]
/// получить кодпоинт по содержащему его блоку и смещению
macro_rules! code_for {
    ($block: expr, $offset: expr) => {
        ($block << BLOCK_BITS) + $offset
    };
}

#[macro_export]
macro_rules! block_for {
    ($code: expr) => {
        $code >> BLOCK_BITS
    };
}

#[test]
fn test_blocks()
{
    let (index, data, ext, stats) = prepare(true);

    let max_block = *index.iter().max().unwrap();

    for i in 0 ..= max_block {
        let mut findex: Vec<usize> = vec![];

        index.iter().enumerate().for_each(|(j, &e)| {
            if e == i {
                findex.push(j);
            }
        });

        if findex.len() != 1 {
            continue;
        }

        println!("данные #{:04X}", i);
        print!("индексы: ");
        findex.iter().for_each(|j| print!("{:04X} ", j));
        println!("\n");

        let block = findex[0];

        for code in code_for!(block, 0) .. code_for!(block + 1, 0) {
            let codepoint = UNICODE.get(&(code as u32));

            let block_offset = (i as usize) << 7;
            let code_offset = ((code as u8) & 0x7F) as usize;

            let value = data[block_offset | code_offset];

            if value == 0 {
                continue;
            }

            match codepoint {
                Some(codepoint) => {
                    let block_name = match codepoint.block {
                        Some(codepoint_block) => codepoint_block.name.clone(),
                        None => "".to_owned(),
                    };

                    println!("U+{:04X} - {} - {}", code, codepoint.name, block_name);
                }
                None => (),
            }
        }

        println!()
    }
}

/// подготавливаем таблицы NFD, NFKD
pub fn prepare<'a>(canonical: bool) -> (Vec<u32>, Vec<u64>, Vec<u32>, CodepointGroups<'a>)
{
    let unicode = &UNICODE;

    let mut index = [0u32; MAX_BLOCKS as usize + 1];
    let mut data: Vec<u64> = vec![];
    let mut expansions = vec![];

    let mut stats = CodepointGroups::new();

    let mut last_block = 0;

    // заполняем блоки

    for block in 0 ..= MAX_BLOCKS {
        let mut block_data = [0u64; 1 << BLOCK_BITS as usize];
        let mut has_contents = code_for!(block, 0) <= STARTING_CODEPOINTS_BLOCK;

        for offset in 0 .. 1 << BLOCK_BITS {
            let code = code_for!(block, offset);

            let codepoint = unicode.get(&code);

            // если кодпоинт не найден - значит это стартер без декомпозиции
            if codepoint.is_none() {
                block_data[offset as usize] = 0;
                continue;
            }

            let codepoint = codepoint.unwrap();

            let (value, expansion) =
                encode_codepoint(codepoint, canonical, expansions.len(), &mut stats);

            if value > 0 {
                has_contents = true;
            }

            expansions.extend(expansion);

            block_data[offset as usize] = value;
        }

        // если в блоке есть данные - его нужно записать
        // в противном случае - индекс должен ссылаться на блок, состоящий из стартеров без декомпозиции.
        // т.к. в блоке 128 значений, то можно ссылаться на 0 блок, где находятся ASCII (у них нет декомпозиции, все CCC = 0)

        if has_contents {
            index[block as usize] = block_for!(data.len()) as u32;
            data.extend(block_data);
            last_block = block;
        }
    }

    // добавляем маркеры для слогов хангыль

    for block in block_for!(0xAC00) .. block_for!(0xD7A3) {
        assert_eq!(index[block], 0);
        index[block] = block_for!(data.len()) as u32;
    }

    data.extend([MARKER_HANGUL; 1 << BLOCK_BITS as usize]);

    index[block_for!(0xD7A3)] = block_for!(data.len()) as u32;

    for offset in 0 .. 1 << BLOCK_BITS {
        let value = match code_for!(block_for!(0xD7A3), offset) <= 0xD7A3 {
            true => MARKER_HANGUL,
            false => 0,
        };

        data.push(value);
    }

    let index = index[0 ..= last_block as usize].to_vec();

    (index, data, expansions, stats)
}
