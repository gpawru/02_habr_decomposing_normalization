use crate::codepoint::Codepoint;

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
    Nonstarter(Codepoint),
    /// декомпозиция на 2 кодпоинта, первый - стартер
    Pair((Codepoint, Codepoint)),
    /// декомпозиция на 3 кодпоинта, первый - стартер
    Triple(Codepoint, Codepoint, Codepoint),
    /// синглтон (стартер, декомпозирующийся в другой стартер)
    Singleton(Codepoint),
    /// декомпозиция на несколько символов, в параметрах - индекс первого элемента в дополнительной таблице и количество этих элементов
    Expansion(u16, u8),
    /// слог хангыль
    Hangul,
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u64) -> DecompositionValue
{
    match value as u8 {
        MARKER_PAIR => parse_pair(value),
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
    DecompositionValue::Nonstarter(Codepoint::from_baked((value >> 8) as u32))
}

/// синглтон
#[inline(always)]
fn parse_singleton(value: u64) -> DecompositionValue
{
    DecompositionValue::Singleton(Codepoint::from_baked((value >> 8) as u32))
}

/// пара
#[inline(always)]
fn parse_pair(value: u64) -> DecompositionValue
{
    DecompositionValue::Pair(unsafe { core::mem::transmute(value & !0xFF) })
}

/// 16-битная тройка
#[inline(always)]
fn parse_triple_16bit(value: u64) -> DecompositionValue
{
    DecompositionValue::Triple(
        Codepoint::from_baked(((value as u16) as u32) << 8),
        Codepoint::from_baked(((value >> 8) as u32) >> 8),
        Codepoint::from_baked((value >> 40) as u32),
    )
}

/// декомпозиция, вынесенная во внешний блок
#[inline(always)]
fn parse_expansion(value: u64) -> DecompositionValue
{
    DecompositionValue::Expansion((value >> 16) as u16, (value >> 8) as u8)
}
