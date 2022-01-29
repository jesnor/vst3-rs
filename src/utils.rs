use std::{
    os::raw::{c_char, c_short},
    ptr::copy_nonoverlapping,
};
use vst3_com::c_void;
use vst3_sys::base::{char16, char8};
use widestring::U16CString;

pub(crate) unsafe fn strcpy(src: &str, dst: *mut c_char) {
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub(crate) unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub(crate) fn char16_to_string(src: &[char16]) -> String {
    let v = src.iter().map(|v| *v as u16).collect::<Vec<_>>();
    String::from_utf16(&v).expect("Invalid string value!")
}

pub(crate) fn string_copy_into_16(src: &str, dst: &mut [char16]) {
    for (i, ch) in src.encode_utf16().enumerate() {
        if i == dst.len() - 1 {
            dst[i] = 0;
            return;
        }

        dst[i] = ch as char16;
    }
}

pub(crate) fn string_to_fixed_width<const LEN: usize>(text: &str) -> [char8; LEN] {
    let mut a = [0; LEN];

    for (i, ch) in text.chars().enumerate() {
        a[i] = ch as char8
    }

    a
}

pub(crate) fn string_to_fixed_width_16<const LEN: usize>(text: &str) -> [char16; LEN] {
    let mut a = [0; LEN];

    for (i, ch) in text.encode_utf16().enumerate() {
        a[i] = ch as char16
    }

    a
}

pub(crate) fn char8_to_16<const LEN: usize>(text: &[char8; LEN]) -> [char16; LEN] {
    let mut a = [0; LEN];

    for i in 0..text.len() {
        a[i] = text[i] as char16
    }

    a
}
