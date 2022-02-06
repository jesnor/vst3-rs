#![allow(dead_code)]

pub mod audio_processor;
pub mod converter;
pub mod edit_controller;
pub mod plugin;
pub mod plugin_parameter;
pub mod range;
pub mod type_cell;
pub mod utils;
mod vst_audio_processor;
mod vst_categories;
mod vst_edit_controller;
pub mod vst_factory;
pub mod vst_stream;

use std::os::raw::c_void;

#[no_mangle]
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "system" fn InitDll() -> bool { true }

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ExitDll() -> bool { true }

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleEntry(_: *mut c_void) -> bool { true }

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleExit() -> bool { true }

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleEntry(_: *mut c_void) -> bool { true }

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleExit() -> bool { true }
