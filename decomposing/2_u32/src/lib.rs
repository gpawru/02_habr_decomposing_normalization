use core::str::from_utf8_unchecked;
use std::marker::PhantomData;

pub use codepoint::Codepoint;
use decomposition::hangul::decompose_hangul_syllable;
use decomposition::hangul::decompose_hangul_syllable_u32;
use decomposition::*;
use slice::aligned::Aligned;
pub use slice::iter::CharsIter;

mod codepoint;
mod data;
mod decomposition;

mod slice;
mod utf8;

pub const NFD: bool = false;
pub const NFKD: bool = true;

/// нормализатор NF(K)D
#[repr(align(32))]
pub struct DecomposingNormalizer<R, const K: bool>
{
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    index: Aligned<'static, u8>,
    /// основные данные
    data: Aligned<'static, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'static, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    ///
    _phantom: PhantomData<R>,
}

/// NFD-нормализатор
pub fn nfd_normalizer() -> DecomposingNormalizer<String, NFD>
{
    from_baked(data::nfd())
}

/// NFD-нормализатор, результат в виде Vec<Codepoint>
pub fn nfd_normalizer_u32() -> DecomposingNormalizer<Vec<Codepoint>, NFD>
{
    from_baked(data::nfd())
}

/// NFKD-нормализатор
pub fn nfkd_normalizer() -> DecomposingNormalizer<String, NFKD>
{
    from_baked(data::nfkd())
}

/// NFKD-нормализатор, результат в виде Vec<Codepoint>
pub fn nfkd_normalizer_u32() -> DecomposingNormalizer<Vec<Codepoint>, NFKD>
{
    from_baked(data::nfkd())
}

/// заранее подготовленные данные
fn from_baked<R, const K: bool>(source: data::DecompositionData) -> DecomposingNormalizer<R, K>
{
    DecomposingNormalizer {
        index: Aligned::from(source.index),
        data: Aligned::from(source.data),
        expansions: Aligned::from(source.expansions),
        continuous_block_end: source.continuous_block_end,
        _phantom: PhantomData,
    }
}

macro_rules! normalizer_methods {
    ($ff_first_boundary: expr, $ff_second_boundary: expr, $rtype:ty, $write: ident, $write_str: ident, $flush: ident, $decompose_hangul: ident) => {
        /// нормализация строки
        /// исходная строка должна являться well-formed UTF-8 строкой
        #[inline(never)]
        pub fn normalize(&self, input: &str) -> $rtype
        {
            let mut result = <$rtype>::with_capacity(input.len());
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
            result: &mut $rtype,
            buffer: &mut Vec<Codepoint>,
        ) -> Option<(u64, u32)>
        {
            Some(loop {
                if iter.is_empty() {
                    $flush(result, buffer);
                    $write_str(result, iter.ending_slice());

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
                        $flush(result, buffer);
                        $write_str(result, iter.block_slice(width));
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
            result: &mut $rtype,
            buffer: &mut Vec<Codepoint>,
        )
        {
            let decomposition = parse_data_value(data_value);

            match decomposition {
                DecompositionValue::Nonstarter(c1) => buffer.push(c1),
                DecompositionValue::Pair((c1, c2)) => {
                    $flush(result, buffer);
                    $write(result, c1);

                    match c2.is_starter() {
                        true => $write(result, c2),
                        false => buffer.push(c2),
                    }
                }
                DecompositionValue::Triple(c1, c2, c3) => {
                    $flush(result, buffer);
                    $write(result, c1);

                    if c3.is_starter() {
                        $write(result, c2);
                        $write(result, c3);
                    } else {
                        match c2.is_starter() {
                            true => $write(result, c2),
                            false => buffer.push(c2),
                        }
                        buffer.push(c3);
                    }
                }
                DecompositionValue::Singleton(c1) => {
                    $flush(result, buffer);
                    $write(result, c1);
                }
                DecompositionValue::Expansion(index, count) => {
                    for entry in
                        &self.expansions[(index as usize) .. (index as usize + count as usize)]
                    {
                        let codepoint = Codepoint::from(*entry);

                        match codepoint.is_starter() {
                            true => {
                                $flush(result, buffer);
                                $write(result, codepoint);
                            }
                            false => buffer.push(codepoint),
                        }
                    }
                }
                DecompositionValue::Hangul => {
                    $flush(result, buffer);
                    $decompose_hangul(result, code);
                }
            }
        }
    };
}

impl<R, const K: bool> DecomposingNormalizer<R, K>
{
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
}

impl DecomposingNormalizer<String, NFD>
{
    normalizer_methods!(
        0xC3,
        0xC0,
        String,
        write_char,
        write_str,
        flush_buffer_str,
        decompose_hangul_syllable
    );
}

impl DecomposingNormalizer<String, NFKD>
{
    normalizer_methods!(
        0xC2,
        0xA0,
        String,
        write_char,
        write_str,
        flush_buffer_str,
        decompose_hangul_syllable
    );
}

impl DecomposingNormalizer<Vec<Codepoint>, NFD>
{
    normalizer_methods!(
        0xC3,
        0xC0,
        Vec<Codepoint>,
        write_u32,
        write_u32_vec,
        flush_buffer_u32,
        decompose_hangul_syllable_u32
    );
}

impl DecomposingNormalizer<Vec<Codepoint>, NFKD>
{
    normalizer_methods!(
        0xC2,
        0xA0,
        Vec<Codepoint>,
        write_u32,
        write_u32_vec,
        flush_buffer_u32,
        decompose_hangul_syllable_u32
    );
}

/// дописать кодпоинт в UTF-8 результат
#[inline(always)]
fn write_char(result: &mut String, codepoint: Codepoint)
{
    result.push(char::from(codepoint));
}

/// дописать кодпоинт как u32
#[inline(always)]
fn write_u32(result: &mut Vec<Codepoint>, codepoint: Codepoint)
{
    result.push(codepoint);
}

/// дописать уже нормализованный кусок исходной строки в UTF-8 результат
#[inline(always)]
fn write_str(result: &mut String, string: &[u8])
{
    result.push_str(unsafe { from_utf8_unchecked(string) });
}

/// дописать уже нормализованный кусок исходной строки в массив кодпоинтов
#[inline(always)]
fn write_u32_vec(result: &mut Vec<Codepoint>, string: &[u8])
{
    unsafe { from_utf8_unchecked(string) }
        .chars()
        .for_each(|c| result.push(Codepoint::from_code(u32::from(c))));
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(always)]
fn flush_buffer_str(result: &mut String, buffer: &mut Vec<Codepoint>)
{
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|codepoint| codepoint.ccc());
        }

        for codepoint in buffer.iter() {
            write_char(result, *codepoint);
        }

        buffer.clear();
    };
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер (u32)
#[inline(always)]
fn flush_buffer_u32(result: &mut Vec<Codepoint>, buffer: &mut Vec<Codepoint>)
{
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|codepoint| codepoint.ccc());
        }

        result.append(buffer);
    };
}
