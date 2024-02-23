use core::str::from_utf8_unchecked;

use codepoint::Codepoint;
use decomposition::hangul::decompose_hangul_syllable;
use decomposition::*;
use slice::aligned::Aligned;
use slice::iter::CharsIter;

mod codepoint;
mod data;
mod decomposition;

mod slice;
mod utf8;

/// нормализатор NF(K)D
#[repr(align(32))]
pub struct DecomposingNormalizer<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    index: Aligned<'a, u8>,
    /// основные данные
    data: Aligned<'a, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'a, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    /// NFD или NFKD
    is_canonical: bool,
}

// методы нормализации вынесены в макрос в целях оптимизации
macro_rules! normalizer_methods {
    ($normalize_method: ident, $ff_method: ident, $ff_first_boundary: expr, $ff_second_boundary: expr) => {
        #[inline(always)]
        fn $normalize_method(&self, input: &str) -> String
        {
            let mut result = String::with_capacity(input.len());
            let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);
            let iter = &mut CharsIter::new(input);

            loop {
                iter.set_breakpoint();

                match self.$ff_method(iter, &mut result, &mut buffer) {
                    Some((data_value, code)) => handle_decomposition_value(
                        data_value,
                        code,
                        &mut result,
                        &mut buffer,
                        &self.expansions,
                    ),
                    None => return result,
                };
            }
        }

        /// цикл быстрой проверки, является-ли часть строки уже нормализованной
        #[inline(always)]
        fn $ff_method(
            &self,
            iter: &mut CharsIter,
            result: &mut String,
            buffer: &mut Vec<Codepoint>,
        ) -> Option<(u64, u32)>
        {
            Some(loop {
                if iter.is_empty() {
                    flush_buffer(result, buffer);

                    result.push_str(unsafe { from_utf8_unchecked(iter.ending_slice()) });

                    return None;
                }

                let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                // все символы до U+00C0 (в NFD) или U+00A0 (NFKD) не имеют декомпозиции
                if first < $ff_first_boundary {
                    continue;
                }

                let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

                if (code & !0xFF == 0) && (code as u8) < $ff_second_boundary {
                    continue;
                }

                // символ за границами "безопасной зоны". проверяем кейс декомпозиции:
                // если он является обычным стартером без декомпозиции, то продолжаем цикл

                let data_value = self.get_decomposition_value(code);

                if data_value != 0 {
                    let width = utf8::get_utf8_sequence_width(first) as isize;

                    // если мы получили какую-то последовательность символов без декомпозиции:
                    //  - сливаем буфер предшествующих этому отрезку нестартеров
                    //  - сливаем отрезок от брейкпоинта до предыдущего символа

                    if !iter.at_breakpoint(width) {
                        flush_buffer(result, buffer);
                        result.push_str(unsafe { from_utf8_unchecked(iter.block_slice(width)) });
                    }

                    break (data_value, code);
                }
            })
        }
    };
}

impl<'a> DecomposingNormalizer<'a>
{
    normalizer_methods!(normalize_nfd, fast_forward_nfd, 0xC3, 0xC0);
    normalizer_methods!(normalize_nfkd, fast_forward_nfkd, 0xC2, 0xA0);

    /// нормализация строки
    /// исходная строка должна являться well-formed UTF-8 строкой
    #[inline(never)]
    pub fn normalize(&self, input: &str) -> String
    {
        match self.is_canonical() {
            true => self.normalize_nfd(input),
            false => self.normalize_nfkd(input),
        }
    }

    /// NFD или NFKD нормализация?
    #[inline(never)]
    fn is_canonical(&self) -> bool
    {
        self.is_canonical
    }

    /// данные о декомпозиции символа
    #[inline(always)]
    fn get_decomposition_value(&self, code: u32) -> u64
    {
        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции
        if code > LAST_DECOMPOSING_CODEPOINT {
            return 0;
        };

        match code <= self.continuous_block_end {
            true => self.data[code as usize],
            false => {
                let block_index = (code >> 7) as usize;
                let block = self.index[block_index] as usize;

                let block_offset = block << 7;
                let code_offset = ((code as u8) & 0x7F) as usize;

                let index = block_offset | code_offset;

                self.data[index]
            }
        }
    }

    /// NFD-нормализатор
    pub fn nfd() -> Self
    {
        Self::from_baked(data::nfd(), true)
    }

    /// NFKD-нормализатор
    pub fn nfkd() -> Self
    {
        Self::from_baked(data::nfkd(), false)
    }

    /// заранее подготовленные данные
    pub fn from_baked(source: data::DecompositionData<'a>, is_canonical: bool) -> Self
    {
        Self {
            index: Aligned::from(source.index),
            data: Aligned::from(source.data),
            expansions: Aligned::from(source.expansions),
            continuous_block_end: source.continuous_block_end,
            is_canonical,
        }
    }
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(never)]
fn flush_buffer(result: &mut String, buffer: &mut Vec<Codepoint>)
{
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|codepoint| codepoint.ccc());
        }

        for codepoint in buffer.iter() {
            result.push(char::from(*codepoint));
        }

        buffer.clear();
    };
}

/// 1. обработать и записать в строку-результат текущее содержимое буфера (кроме случая с нестартерами),
/// 2. записать / дописать в буфер декомпозицию кодпоинта (стартер - сразу в результат)
#[inline(always)]
fn handle_decomposition_value(
    data_value: u64,
    code: u32,
    result: &mut String,
    buffer: &mut Vec<Codepoint>,
    expansions: &[u32],
)
{
    let decomposition = parse_data_value(data_value);

    match decomposition {
        DecompositionValue::Nonstarter(c1) => buffer.push(c1),
        DecompositionValue::Pair((c1, c2)) => {
            flush_buffer(result, buffer);
            result.push(char::from(c1));

            match c2.is_starter() {
                true => result.push(char::from(c2)),
                false => buffer.push(c2),
            }
        }
        DecompositionValue::Triple(c1, c2, c3) => {
            flush_buffer(result, buffer);
            result.push(char::from(c1));

            if c3.is_starter() {
                result.push(char::from(c2));
                result.push(char::from(c3));
            } else {
                match c2.is_starter() {
                    true => result.push(char::from(c2)),
                    false => buffer.push(c2),
                }
                buffer.push(c3);
            }
        }
        DecompositionValue::Singleton(c1) => {
            flush_buffer(result, buffer);
            result.push(char::from(c1));
        }
        DecompositionValue::Expansion(index, count) => {
            for entry in &expansions[(index as usize) .. (index as usize + count as usize)] {
                let codepoint = Codepoint::from(*entry);

                match codepoint.is_starter() {
                    true => {
                        flush_buffer(result, buffer);
                        result.push(char::from(codepoint));
                    }
                    false => buffer.push(codepoint),
                }
            }
        }
        DecompositionValue::Hangul => {
            flush_buffer(result, buffer);
            decompose_hangul_syllable(result, code);
        }
    }
}
