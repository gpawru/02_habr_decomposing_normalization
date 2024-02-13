use crate::o;

pub mod hangul;

/// последний кодпоинт с декомпозицией
pub const LAST_DECOMPOSING_CODEPOINT: u32 = 0x2FA1D;

/// нестартер без декомпозиции
const MARKER_NONSTARTER: u8 = 1;
/// 16-битная пара
const MARKER_PAIR: u8 = 2;
/// синглтон
const MARKER_SINGLETON: u8 = 3;
/// декомпозиция, вынесенная во внешний блок
const MARKER_EXPANSION: u8 = 4;
/// слог хангыль
const MARKER_HANGUL: u8 = 5;

pub enum DecompositionValue
{
    /// нестартер (например, диакретический знак)
    Nonstarter(u8),
    /// декомпозиция на 2 кодпоинта, первый - стартер
    Pair(u32, Codepoint),
    /// декомпозиция на 3 кодпоинта, первый - стартер
    Triple(u32, Codepoint, Codepoint),
    /// синглтон (стартер, декомпозирующийся в другой стартер)
    Singleton(u32),
    /// декомпозиция на несколько символов, в параметрах - индекс первого элемента в дополнительной таблице и количество этих элементов
    Expansion(u16, u8),
    /// слог хангыль
    Hangul,
}

/// кодпоинт для декомпозиции
pub struct Codepoint
{
    /// класс комбинирования
    pub ccc: u8,
    /// код символа
    pub code: u32,
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u64) -> DecompositionValue
{
    match value as u8 {
        MARKER_PAIR => parse_pair_16bit(value),
        MARKER_NONSTARTER => parse_nonstarter(value),
        MARKER_SINGLETON => parse_singleton(value),
        MARKER_EXPANSION => parse_expansion(value),
        MARKER_HANGUL => DecompositionValue::Hangul,
        _ => parse_triple_16bit(value),
    }
}

/// нестартер без декомпозиции
#[inline(always)]
fn parse_nonstarter(value: u64) -> DecompositionValue
{
    DecompositionValue::Nonstarter(o!(value, u8, 1))
}

/// синглтон
#[inline(always)]
fn parse_singleton(value: u64) -> DecompositionValue
{
    DecompositionValue::Singleton(o!(value, u32, 1))
}

/// 16-битная пара
#[inline(always)]
fn parse_pair_16bit(value: u64) -> DecompositionValue
{
    DecompositionValue::Pair(
        o!(value, u16, 1) as u32,
        Codepoint {
            ccc: o!(value, u8, 1),
            code: o!(value, u16, 2) as u32,
        },
    )
}

/// 16-битная тройка
#[inline(always)]
fn parse_triple_16bit(value: u64) -> DecompositionValue
{
    DecompositionValue::Triple(
        o!(value, u16) as u32,
        Codepoint {
            code: o!(value, u16, 1) as u32,
            ccc: o!(value, u8, 6),
        },
        Codepoint {
            code: o!(value, u16, 2) as u32,
            ccc: o!(value, u8, 7),
        },
    )
}

/// декомпозиция, вынесенная во внешний блок
#[inline(always)]
fn parse_expansion(value: u64) -> DecompositionValue
{
    DecompositionValue::Expansion(o!(value, u16, 1), o!(value, u8, 1))
}
