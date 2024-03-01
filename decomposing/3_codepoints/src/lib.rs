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

/// последний кодпоинт с декомпозицией
pub const LAST_DECOMPOSING_CODEPOINT: u32 = 0x2FA1D;

/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u8 = 1;
/// 16-битная пара
pub const MARKER_PAIR: u8 = 2;
/// декомпозиция, вынесенная во внешний блок
pub const MARKER_EXPANSION: u8 = 3;
/// синглтон
pub const MARKER_SINGLETON: u8 = 4;
/// слог хангыль
pub const MARKER_HANGUL: u8 = 5;

// нормализатор NF(K)D
#[repr(C, align(16))]
pub struct DecomposingNormalizer
{
    /// основные данные
    data: Aligned<'static, u64>,
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
    pub fn normalize(&self, input: &str) -> Vec<Codepoint>
    {
        let mut result = Vec::with_capacity(input.len());
        let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);
        let iter = &mut CharsIter::new(input);

        loop {
            let entry: Option<(u64, u32)> = match !buffer.is_empty() {
                true => match self.forward(iter, &mut result, &mut buffer) {
                    Some(entry) => Some(entry),
                    None => continue,
                },
                false => self.fast_forward(iter, &mut result),
            };

            match entry {
                Some((data_value, code)) => {
                    self.handle_decomposition_value(data_value, code, &mut result, &mut buffer)
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
        result: &mut Vec<Codepoint>,
        buffer: &mut Vec<Codepoint>,
    ) -> Option<(u64, u32)>
    {
        if iter.is_empty() {
            flush_inline(result, buffer);
            return None;
        }

        let first = unsafe { utf8::char_first_byte_unchecked(iter) };

        if first < 0x80 {
            flush_inline(result, buffer);
            write(result, Codepoint::from_code(first as u32));

            return None;
        }

        let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

        if first < 0xC2 {
            flush_inline(result, buffer);
            write(result, Codepoint::from_code(code));

            return None;
        }

        let data_value = self.get_decomposition_value(code);

        match data_value != 0 {
            true => Some((data_value, code)),
            false => {
                flush_inline(result, buffer);
                write(result, Codepoint::from_code(code));

                None
            }
        }
    }

    /// цикл быстрой проверки, является-ли часть строки уже нормализованной
    #[inline(always)]
    fn fast_forward(&self, iter: &mut CharsIter, result: &mut Vec<Codepoint>)
        -> Option<(u64, u32)>
    {
        loop {
            if iter.is_empty() {
                return None;
            }

            let first = unsafe { utf8::char_first_byte_unchecked(iter) };

            if first < 0x80 {
                write(result, Codepoint::from_code(first as u32));
                continue;
            }

            let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

            if first < 0xC2 {
                write(result, Codepoint::from_code(code));
                continue;
            }

            // символ за границами "безопасной зоны". проверяем кейс декомпозиции:
            // если он является обычным стартером без декомпозиции, то продолжаем цикл

            let data_value = self.get_decomposition_value(code);

            if data_value != 0 {
                return Some((data_value, code));
            }

            write(result, Codepoint::from_code(code));
        }
    }

    /// записать / дописать в буфер декомпозицию кодпоинта (стартер - сразу в результат)
    #[inline(always)]
    fn handle_decomposition_value(
        &self,
        value: u64,
        code: u32,
        result: &mut Vec<Codepoint>,
        buffer: &mut Vec<Codepoint>,
    )
    {
        match value as u8 {
            MARKER_NONSTARTER => {
                buffer.push(Codepoint::from_baked((value >> 8) as u32));
            }
            MARKER_PAIR => {
                flush_inline(result, buffer);
                write(result, Codepoint::from_baked((value as u32) & !0xFF));

                let c2 = Codepoint::from_baked((value >> 32) as u32);

                match c2.is_starter() {
                    true => write(result, c2),
                    false => buffer.push(c2),
                }
            }
            MARKER_SINGLETON => {
                flush(result, buffer);
                write(result, Codepoint::from_baked((value >> 8) as u32));
            }
            MARKER_EXPANSION => {
                let index = (value >> 16) as u16;
                let count = (value >> 8) as u8;

                for &entry in
                    &self.expansions[(index as usize) .. (index as usize + count as usize)]
                {
                    let codepoint = Codepoint::from_baked(entry);

                    match codepoint.is_starter() {
                        true => {
                            flush(result, buffer);
                            write(result, codepoint);
                        }
                        false => buffer.push(codepoint),
                    }
                }
            }
            MARKER_HANGUL => {
                flush(result, buffer);
                decompose_hangul_syllable(result, code);
            }
            _ => {
                flush(result, buffer);
                write(result, Codepoint::from_baked(((value as u16) as u32) << 8));

                let c2 = Codepoint::from_baked(((value >> 8) as u32) >> 8);
                let c3 = Codepoint::from_baked((value >> 40) as u32);

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
}

/// дописать кодпоинт в результат
#[inline(always)]
fn write(result: &mut Vec<Codepoint>, codepoint: Codepoint)
{
    result.push(codepoint);
}

// Rust Perfomance Book утверждает, что такой подход к инлайну - это нормально :(

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(never)]
fn flush(result: &mut Vec<Codepoint>, buffer: &mut Vec<Codepoint>)
{
    flush_inline(result, buffer)
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(always)]
fn flush_inline(result: &mut Vec<Codepoint>, buffer: &mut Vec<Codepoint>)
{
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|codepoint| codepoint.ccc());
        }

        result.append(buffer);
    };
}
