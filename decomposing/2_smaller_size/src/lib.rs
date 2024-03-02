use core::str::from_utf8_unchecked;

pub use codepoint::Codepoint;
pub use data::DecompositionData;
use hangul::decompose_hangul_syllable;
use slice::aligned::Aligned;
pub use slice::iter::CharsIter;

mod codepoint;
mod data;
mod hangul;
mod slice;
mod utf8;

/// последний кодпоинт с декомпозицией (U+2FA1D), его блок - 0x5F4
pub const LAST_DECOMPOSING_CODEPOINT_BLOCK: u16 = 0x5F4;

/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u8 = 1;
/// синглтон
pub const MARKER_SINGLETON: u8 = 2;
/// декомпозиция, вынесенная во внешний блок
pub const MARKER_EXPANSION: u8 = 3;
/// слог хангыль
pub const MARKER_HANGUL: u8 = 4;

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
}

/// заранее подготовленные данные
pub fn from_baked(source: DecompositionData) -> DecomposingNormalizer
{
    DecomposingNormalizer {
        data: Aligned::from(source.data),
        index: Aligned::from(source.index),
        expansions: Aligned::from(source.expansions),
        continuous_block_end: source.continuous_block_end,
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
            let entry = match !buffer.is_empty() {
                true => match self.forward(iter, &mut result, &mut buffer) {
                    Some(entry) => Some(entry),
                    None => continue,
                },
                false => self.fast_forward(iter, &mut result),
            };

            match entry {
                Some((data_value, code)) => {
                    self.handle_decomposition_value(data_value, code, &mut result, &mut buffer);
                    iter.set_breakpoint();
                }
                None => return result,
            }
        }
    }

    /// если буфер не пуст, мы не можем перейти к быстрой проверке.
    /// прочитаем следующий кодпоинт, и если он стартер - скомбинируем буфер
    #[inline(always)]
    fn forward(
        &self,
        iter: &mut CharsIter,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    ) -> Option<(u32, u32)>
    {
        iter.set_breakpoint();

        if !iter.is_empty() {
            let first = unsafe { utf8::char_first_byte_unchecked(iter) };

            if first >= 0xC2 {
                let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };
                let data_value = self.get_decomposition_value(code);

                if data_value != 0 {
                    return Some((data_value, code));
                }
            }
        }

        flush_inline(result, buffer);
        None
    }

    /// цикл быстрой проверки, является-ли часть строки уже нормализованной
    #[inline(always)]
    fn fast_forward(&self, iter: &mut CharsIter, result: &mut String) -> Option<(u32, u32)>
    {
        loop {
            if iter.is_empty() {
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

            if data_value == 0 {
                continue;
            }

            let width = utf8::get_utf8_sequence_width(first) as isize;

            // если мы получили какую-то последовательность символов без декомпозиции:
            //  - сливаем буфер предшествующих этому отрезку нестартеров
            //  - сливаем отрезок от брейкпоинта до предыдущего символа

            if !iter.at_breakpoint(width) {
                write_str(result, iter.block_slice(width));
            }

            return Some((data_value, code));
        }
    }

    /// 1. обработать и записать в строку-результат текущее содержимое буфера (кроме случая с нестартерами),
    /// 2. записать / дописать в буфер декомпозицию кодпоинта (стартер - сразу в результат)
    #[inline(always)]
    fn handle_decomposition_value(
        &self,
        value: u32,
        code: u32,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        let marker = value as u8;

        match marker {
            MARKER_NONSTARTER => {
                buffer.push(Codepoint::from_code_and_ccc(code, (value >> 8) as u8))
            }
            MARKER_SINGLETON => {
                flush(result, buffer);
                write_char(result, (value >> 8) as u32);
            }
            MARKER_EXPANSION => {
                let index = (value >> 16) as u16;
                let count = (value >> 8) as u8;

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
            MARKER_HANGUL => {
                flush(result, buffer);
                decompose_hangul_syllable(result, code);
            }
            _ => {
                flush(result, buffer);
                write_char(result, (value as u16) as u32);

                let c2 = value >> 16;
                let ccc = (self.get_decomposition_value(c2) >> 8) as u8;

                match ccc != 0 {
                    true => buffer.push(Codepoint::from_code_and_ccc(c2, ccc)),
                    false => write_char(result, c2 as u32),
                }
            }
        }
    }

    /// данные о декомпозиции символа
    #[inline(always)]
    fn get_decomposition_value(&self, code: u32) -> u32
    {
        if code <= self.continuous_block_end {
            return self.data[code as usize];
        }

        let block_index = (code >> 7) as u16;

        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции
        if block_index > LAST_DECOMPOSING_CODEPOINT_BLOCK {
            return 0;
        };

        let block_index = (code >> 7) as usize;
        let block = self.index[block_index] as usize;

        let block_offset = block << 7;
        let code_offset = ((code as u8) & 0x7F) as usize;

        let index = block_offset | code_offset;

        self.data[index]
    }
}

/// не-инлайн вариант функции
#[inline(never)]
fn flush(result: &mut String, buffer: &mut Vec<Codepoint>)
{
    flush_inline(result, buffer)
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(always)]
fn flush_inline(result: &mut String, buffer: &mut Vec<Codepoint>)
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
