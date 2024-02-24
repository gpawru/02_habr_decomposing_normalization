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
    data: Aligned<'static, u64>,
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    index: Aligned<'static, u8>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'static, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    _p1: [u32; 3],
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
    ) -> Option<(u64, u32)>
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
    #[inline(always)]
    fn handle_decomposition_value(
        &self,
        data_value: u64,
        code: u32,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        let decomposition = parse_data_value(data_value);

        match decomposition {
            DecompositionValue::Nonstarter(c1) => buffer.push(c1),
            DecompositionValue::Pair((c1, c2)) => {
                flush(result, buffer);
                write(result, c1);

                match c2.is_starter() {
                    true => write(result, c2),
                    false => buffer.push(c2),
                }
            }
            DecompositionValue::Triple(c1, c2, c3) => {
                flush(result, buffer);
                write(result, c1);

                if c3.is_starter() {
                    write(result, c2);
                    write(result, c3);
                } else {
                    match c2.is_starter() {
                        true => write(result, c2),
                        false => buffer.push(c2),
                    }
                    buffer.push(c3);
                }
            }
            DecompositionValue::Singleton(c1) => {
                flush(result, buffer);
                write(result, c1);
            }
            DecompositionValue::Expansion(index, count) => {
                for entry in &self.expansions[(index as usize) .. (index as usize + count as usize)]
                {
                    let codepoint = Codepoint::from(*entry);

                    match codepoint.is_starter() {
                        true => {
                            flush(result, buffer);
                            write(result, codepoint);
                        }
                        false => buffer.push(codepoint),
                    }
                }
            }
            DecompositionValue::Hangul => {
                flush(result, buffer);
                decompose_hangul_syllable(result, code);
            }
        }
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
    pub fn nfd() -> DecomposingNormalizer
    {
        Self::from_baked(data::nfd())
    }

    /// NFKD-нормализатор
    pub fn nfkd() -> DecomposingNormalizer
    {
        Self::from_baked(data::nfkd())
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

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(always)]
fn flush(result: &mut String, buffer: &mut Vec<Codepoint>)
{
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|codepoint| codepoint.ccc());
        }

        for codepoint in buffer.iter() {
            write(result, *codepoint);
        }

        buffer.clear();
    };
}
