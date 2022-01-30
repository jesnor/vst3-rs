#![allow(dead_code)]

mod audio_processor;
mod converter;
mod edit_controller;
mod plugin;
mod plugin_parameter;
mod range;
mod sine_synth;
mod type_cell;
mod utils;
mod vst_audio_processor;
mod vst_categories;
mod vst_edit_controller;
mod vst_factory;
mod vst_stream;

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
