use vst3_sys::base::{char16, char8};

pub(crate) fn char16_to_string(src: &[char16]) -> String {
    let v = src.iter().map(|v| *v as u16).collect::<Vec<_>>();
    String::from_utf16(&v).expect("Invalid string value!")
}

pub(crate) fn string_copy_into_i16(src: &str, dst: &mut [i16]) {
    let mut i = 0;

    for ch in src.encode_utf16() {
        if i == dst.len() - 1 {
            break;
        }

        dst[i] = ch as char16;
        i += 1;
    }

    dst[i] = 0;
}

pub(crate) fn string_copy_into_u16(src: &str, dst: &mut [u16]) {
    let mut i = 0;

    for ch in src.encode_utf16() {
        if i == dst.len() - 1 {
            break;
        }

        dst[i] = ch;
        i += 1;
    }

    dst[i] = 0;
}

pub(crate) fn string_to_fixed_width<const LEN: usize>(text: &str) -> [char8; LEN] {
    let mut a = [0; LEN];

    for (i, ch) in text.chars().enumerate() {
        a[i] = ch as char8
    }

    a
}

pub(crate) fn string_to_fixed_width_i16<const LEN: usize>(text: &str) -> [char16; LEN] {
    let mut a = [0; LEN];
    string_copy_into_i16(text, &mut a);
    a
}

pub(crate) fn char8_to_16<const LEN: usize>(text: &[char8; LEN]) -> [char16; LEN] {
    let mut a = [0; LEN];

    for i in 0..text.len() {
        a[i] = text[i] as char16
    }

    a
}
