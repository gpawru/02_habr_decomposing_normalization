use crate::Codepoint;

/// начало блока слогов хангыль
const HANGUL_S_BASE: u32 = 0xAC00;
/// начало блока ведущих согласных чамо
pub const HANGUL_L_BASE: u32 = 0x1100;
/// начало блока гласных чамо
pub const HANGUL_V_BASE: u32 = 0x1161;
/// начало блока завершающих согласных (на 1 меньше, см. спецификацию)
pub const HANGUL_T_BASE: u32 = 0x11A7;
/// количество гласных * количество завершающих согласных
const HANGUL_N_COUNT: u32 = 588;
/// количество завершающих согласных
const HANGUL_T_COUNT: u32 = 27;
/// количество кодпоинтов на блок LV
const HANGUL_T_BLOCK_SIZE: u32 = HANGUL_T_COUNT + 1;

/// декомпозиция слога хангыль
#[inline(always)]
pub fn decompose_hangul_syllable(result: &mut Vec<Codepoint>, code: u32)
{
    let lvt = code - HANGUL_S_BASE;

    let l = lvt / HANGUL_N_COUNT;
    let v = (lvt % HANGUL_N_COUNT) / HANGUL_T_BLOCK_SIZE;
    let t = lvt % HANGUL_T_BLOCK_SIZE;

    let c0 = HANGUL_L_BASE + l;
    let c1 = HANGUL_V_BASE + v;

    macro_rules! write_codepoints {
        ($($code: expr),+) => {
            $(result.push(Codepoint::from_code($code));)+
        };
    }

    match t == 0 {
        true => {
            write_codepoints!(c0, c1);
        }
        false => {
            let c2 = HANGUL_T_BASE + t;
            write_codepoints!(c0, c1, c2);
        }
    }
}
