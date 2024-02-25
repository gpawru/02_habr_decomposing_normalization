/// кодпоинт для декомпозиции в виде u32, где CCC хранится в старших битах
#[derive(Debug, Clone, Copy)]
pub struct Codepoint(u32);

impl From<Codepoint> for char
{
    #[inline(always)]
    fn from(value: Codepoint) -> Self
    {
        unsafe { char::from_u32_unchecked(value.0 >> 8) }
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
    pub fn from_baked(code: u32) -> Self
    {
        Self(code)
    }

    #[inline(always)]
    pub fn from_code_and_ccc(code: u32, ccc: u8) -> Self
    {
        Self(code << 8 | (ccc as u32))
    }

    #[inline(always)]
    pub fn from_code(code: u32) -> Self
    {
        Self(code << 8)
    }
}
