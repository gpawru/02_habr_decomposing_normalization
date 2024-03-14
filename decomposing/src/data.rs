/// структура хранимых данных для нормализации
pub struct DecompositionData<'a>
{
    /// индекс блока
    pub index: &'a [u16],
    /// основные данные
    pub data: &'a [u32],
    /// данные кодпоинтов, которые не вписываются в основную часть
    pub expansions: &'a [u32],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    pub continuous_block_end: u32,
}

/// данные для NFD-нормализации
pub fn nfd<'a>() -> DecompositionData<'a>
{
    include!("./../../data/nfd.txt")
}

/// данные для NFKD-нормализации
pub fn nfkd<'a>() -> DecompositionData<'a>
{
    include!("./../../data/nfkd.txt")
}
