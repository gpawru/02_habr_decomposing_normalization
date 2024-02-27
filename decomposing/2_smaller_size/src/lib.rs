use core::str::from_utf8_unchecked;

pub use codepoint::Codepoint;
pub use data::DecompositionData;
use decomposition::hangul::decompose_hangul_syllable;
use decomposition::*;
use slice::aligned::Aligned;
pub use slice::iter::CharsIter;

mod codepoint;
mod data;
mod decomposition;

mod slice;
mod utf8;

// нормализатор NF(K)D
#[repr(C, align(16))]
pub struct DecomposingNormalizer
{
    /// основные данные
    data: Aligned<'static, u32>,
    /// индекс блока
    index: Aligned<'static, u8>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'static, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    _p1: [u32; 3],
}
/// заранее подготовленные данные
pub fn from_baked(source: DecompositionData) -> DecomposingNormalizer
{
    DecomposingNormalizer {
        data: Aligned::from(source.data),
        index: Aligned::from(source.index),
        expansions: Aligned::from(source.expansions),
        continuous_block_end: source.continuous_block_end,
        _p1: [0; 3],
    }
}

/// NFD-нормализатор
pub fn new_nfd() -> DecomposingNormalizer
{
    from_baked(data::nfd())
}

/// NFKD-нормализатор
pub fn new_nfkd() -> DecomposingNormalizer
{
    from_baked(data::nfkd())
}

impl DecomposingNormalizer
{
    /// нормализация строки
    /// исходная строка должна являться well-formed UTF-8 строкой
    #[inline(never)]
    pub fn normalize(&self, input: &str) -> String
    {
        let mut result = String::with_capacity(input.len());
        let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);
        let iter = &mut CharsIter::new(input);

        loop {
            iter.set_breakpoint();

            match self.fast_forward(iter, &mut result, &mut buffer) {
                Some((data_value, code)) => {
                    self.handle_decomposition_value(data_value, code, &mut result, &mut buffer)
                }
                None => return result,
            };
        }
    }

    /// цикл быстрой проверки, является-ли часть строки уже нормализованной
    #[inline(always)]
    fn fast_forward(
        &self,
        iter: &mut CharsIter,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    ) -> Option<(u32, u32)>
    {
        Some(loop {
            if iter.is_empty() {
                flush(result, buffer);
                write_str(result, iter.ending_slice());

                return None;
            }

            let first = unsafe { utf8::char_first_byte_unchecked(iter) };

            // все символы до U+00C0 (в NFD) или U+00A0 (NFKD) не имеют декомпозиции,
            // первый байт UTF-8 U+00A0 (как наименьшего) - 0xC2. тогда получается, что в
            // следующий раз нам встретится 2й байт последовательности со старшими битами 0b10,
            // что заведомо меньше, чем 0b11 для 0xC2, и этот байт также будет пропущен.

            if first < 0xC2 {
                continue;
            }

            let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

            // символ за границами "безопасной зоны". проверяем кейс декомпозиции:
            // если он является обычным стартером без декомпозиции, то продолжаем цикл

            let data_value = self.get_decomposition_value(code);

            if data_value != 0 {
                let width = utf8::get_utf8_sequence_width(first) as isize;

                // если мы получили какую-то последовательность символов без декомпозиции:
                //  - сливаем буфер предшествующих этому отрезку нестартеров
                //  - сливаем отрезок от брейкпоинта до предыдущего символа

                if !iter.at_breakpoint(width) {
                    flush(result, buffer);
                    write_str(result, iter.block_slice(width));
                }

                break (data_value, code);
            }
        })
    }

    /// 1. обработать и записать в строку-результат текущее содержимое буфера (кроме случая с нестартерами),
    /// 2. записать / дописать в буфер декомпозицию кодпоинта (стартер - сразу в результат)
    #[inline(never)]
    fn handle_decomposition_value(
        &self,
        data_value: u32,
        code: u32,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        match data_value as u8 {
            1 | 3 => (),
            _ => flush(result, buffer),
        }

        let decomposition = parse_data_value(data_value);

        match decomposition {
            DecompositionValue::Pair((c1, c2)) => {
                write_char(result, c1 as u32);

                let c2 = c2 as u32;
                let ccc = (self.get_decomposition_value(c2) >> 8) as u8;

                match ccc != 0 {
                    true => buffer.push(Codepoint::from_code_and_ccc(c2, ccc)),
                    false => write_char(result, c2 as u32),
                }
            }
            DecompositionValue::Nonstarter(ccc) => {
                buffer.push(Codepoint::from_code_and_ccc(code, ccc))
            }

            DecompositionValue::Singleton(code) => {
                write_char(result, code);
            }
            DecompositionValue::Expansion(index, count) => {
                for &entry in
                    &self.expansions[(index as usize) .. (index as usize + count as usize)]
                {
                    match entry as u8 != 0 {
                        true => {
                            buffer.push(Codepoint::from_baked(entry));
                        }
                        false => {
                            flush(result, buffer);
                            write_char(result, entry >> 8);
                        }
                    }
                }
            }
            DecompositionValue::Hangul => {
                decompose_hangul_syllable(result, code);
            }
        }
    }

    /// данные о декомпозиции символа
    #[inline]
    fn get_decomposition_value(&self, code: u32) -> u32
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
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline]
fn flush(result: &mut String, buffer: &mut Vec<Codepoint>)
{
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|codepoint| codepoint.ccc());
        }

        for &codepoint in buffer.iter() {
            write(result, codepoint);
        }

        buffer.clear();
    };
}

/// дописать символ(по коду) в результат
#[inline(always)]
fn write_char(result: &mut String, code: u32)
{
    result.push(unsafe { char::from_u32_unchecked(code) });
}

/// дописать кодпоинт в UTF-8 результат
#[inline(always)]
fn write(result: &mut String, codepoint: Codepoint)
{
    result.push(char::from(codepoint));
}

/// дописать уже нормализованный кусок исходной строки в UTF-8 результат
#[inline(always)]
fn write_str(result: &mut String, string: &[u8])
{
    result.push_str(unsafe { from_utf8_unchecked(string) });
}
