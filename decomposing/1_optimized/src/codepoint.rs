/// кодпоинт для декомпозиции в виде u32, где CCC хранится в старших битах
#[derive(Debug, Clone, Copy)]
pub struct Codepoint(u32);

impl From<u32> for Codepoint
{
    #[inline]
    fn from(value: u32) -> Self
    {
        Self(value)
    }
}

impl From<Codepoint> for u32
{
    #[inline]
    fn from(value: Codepoint) -> Self
    {
        value.0
    }
}

impl From<Codepoint> for char
{
    #[inline]
    fn from(value: Codepoint) -> Self
    {
        unsafe { char::from_u32_unchecked(value.code()) }
    }
}

impl Codepoint
{
    #[inline(always)]
    pub fn code(&self) -> u32
    {
        self.0 >> 8
    }

    #[inline(always)]
    pub fn ccc(&self) -> u8
    {
        self.0 as u8
    }

    #[inline(always)]
    pub fn is_starter(&self) -> bool
    {
        self.0 as u8 == 0
    }

    #[inline(always)]
    pub fn is_nonstarter(&self) -> bool
    {
        self.0 as u8 != 0
    }

    #[inline(always)]
    pub fn compose(code: u32, ccc: u8) -> Self
    {
        Self(code << 8 | (ccc as u32))
    }
}
