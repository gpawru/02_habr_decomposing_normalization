use core::str::from_utf8_unchecked;

/// начало блока слогов хангыль
const HANGUL_S_BASE: u32 = 0xAC00;
/// количество гласных * количество завершающих согласных
const HANGUL_N_COUNT: u32 = 588;
/// количество завершающих согласных
const HANGUL_T_COUNT: u32 = 27;
/// количество кодпоинтов на блок LV
const HANGUL_T_BLOCK_SIZE: u32 = HANGUL_T_COUNT + 1;

/// декомпозиция слога хангыль
#[inline(never)]
pub fn decompose_hangul_syllable(result: &mut String, code: u32)
{
    let lvt = code.wrapping_sub(HANGUL_S_BASE);

    let l = (lvt / HANGUL_N_COUNT) as u8;
    let v = ((lvt % HANGUL_N_COUNT) / HANGUL_T_BLOCK_SIZE) as u8;
    let t = (lvt % HANGUL_T_BLOCK_SIZE) as u8;

    let c0 = 0x80 + l;
    let c1 = 0xA1 + v;

    match t == 0 {
        true => result.push_str(unsafe { from_utf8_unchecked(&[0xE1, 0x84, c0, 0xE1, 0x85, c1]) }),
        false => {
            let c2 = 0x86 | ((0x07 + t) >> 5);
            let c3 = 0x80 | ((0xA7 + t) & 0x3F);

            result.push_str(unsafe {
                from_utf8_unchecked(&[0xE1, 0x84, c0, 0xE1, 0x85, c1, 0xE1, c2, c3])
            });
        }
    };
}
