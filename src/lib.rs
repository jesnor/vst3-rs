mod plugin;
mod sine_synth;
mod utils;
mod vst_audio_processor;
mod vst_categories;
mod vst_factory;

use crate::utils::wstrcpy;
use std::cell::RefCell;
use std::mem;
use std::os::raw::c_void;
use std::ptr::null_mut;
use vst3_com::sys::GUID;
use vst3_com::ComPtr;
use vst3_sys::base::{kResultFalse, kResultOk, kResultTrue, tresult, FIDString, IBStream, IPluginBase, IUnknown};
use vst3_sys::utils::VstPtr;
use vst3_sys::vst::ParameterFlags::{kCanAutomate, kIsBypass};
use vst3_sys::vst::{IComponentHandler, IEditController, IUnitInfo, ParameterInfo, ProgramListInfo, TChar, UnitInfo};
use vst3_sys::VST3;

struct Units(Vec<UnitInfo>);
struct Parameters(Vec<(ParameterInfo, f64)>);
struct ComponentHandler(*mut c_void);

struct ContextPtr(*mut c_void);

#[VST3(implements(IEditController, IUnitInfo))]
pub struct AGainController {
    units:             RefCell<Units>,
    parameters:        RefCell<Parameters>,
    context:           RefCell<ContextPtr>,
    component_handler: RefCell<ComponentHandler>,
}

impl AGainController {
    const CID: GUID = GUID {
        data: [
            0xD3, 0x9D, 0x5B, 0x65, 0xD7, 0xAF, 0x42, 0xFA, 0x84, 0x3F, 0x4A, 0xC8, 0x41, 0xEB, 0x04, 0xF0,
        ],
    };
    pub fn new() -> Box<Self> {
        let units = RefCell::new(Units(vec![]));
        let parameters = RefCell::new(Parameters(vec![]));
        let context = RefCell::new(ContextPtr(null_mut()));
        let component_handler = RefCell::new(ComponentHandler(null_mut()));
        AGainController::allocate(units, parameters, context, component_handler)
    }

    pub fn create_instance() -> *mut c_void { Box::into_raw(Self::new()) as *mut c_void }
}

impl IEditController for AGainController {
    unsafe fn set_component_state(&self, state: *mut c_void) -> tresult {
        if state.is_null() {
            return kResultFalse;
        }

        let state = state as *mut *mut _;
        let state: ComPtr<dyn IBStream> = ComPtr::new(state);

        let mut num_bytes_read = 0;
        let mut saved_gain = 0.0;
        let mut saved_bypass = false;
        let gain_ptr = &mut saved_gain as *mut f64 as *mut c_void;
        let bypass_ptr = &mut saved_bypass as *mut bool as *mut c_void;

        state.read(gain_ptr, mem::size_of::<f64>() as i32, &mut num_bytes_read);

        state.read(bypass_ptr, mem::size_of::<bool>() as i32, &mut num_bytes_read);

        self.set_param_normalized(0, saved_gain);
        self.set_param_normalized(1, if saved_bypass { 1.0 } else { 0.0 });

        kResultOk
    }
    unsafe fn set_state(&self, _state: *mut c_void) -> tresult { kResultOk }
    unsafe fn get_state(&self, _state: *mut c_void) -> tresult { kResultOk }
    unsafe fn get_parameter_count(&self) -> i32 { self.parameters.borrow().0.len() as i32 }
    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
        if param_index >= 0 && param_index < self.parameters.borrow().0.len() as i32 {
            *info = self.parameters.borrow().0[param_index as usize].0;
            return kResultTrue;
        }

        kResultFalse
    }
    unsafe fn get_param_string_by_value(&self, id: u32, value_normalized: f64, string: *mut TChar) -> tresult {
        match id {
            0 => {
                let value = format!("{:.0}", value_normalized * 100.0);
                wstrcpy(&value, string);
                kResultTrue
            }

            _ => kResultFalse,
        }
    }
    unsafe fn get_param_value_by_string(
        &self,
        _id: u32,
        _string: *const TChar,
        _value_normalized: *mut f64,
    ) -> tresult {
        kResultFalse
    }
    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        match id {
            0 => value_normalized * 100.0,
            1 => value_normalized,
            _ => unreachable!(),
        }
    }
    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        match id {
            0 => plain_value / 100.0,
            1 => plain_value,
            _ => unreachable!(),
        }
    }
    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        match id {
            0 => self.parameters.borrow().0[0].1,
            1 => self.parameters.borrow().0[1].1,
            _ => unreachable!(),
        }
    }
    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        match id {
            0 => {
                self.parameters.borrow_mut().0[0].1 = value;
                kResultTrue
            }
            1 => {
                self.parameters.borrow_mut().0[1].1 = value;
                kResultTrue
            }
            _ => kResultFalse,
        }
    }
    unsafe fn set_component_handler(&self, handler: *mut c_void) -> tresult {
        if self.component_handler.borrow().0 == handler {
            return kResultTrue;
        }

        if !self.component_handler.borrow().0.is_null() {
            let component_handler = self.component_handler.borrow_mut().0 as *mut *mut _;
            let component_handler: ComPtr<dyn IComponentHandler> = ComPtr::new(component_handler);
            component_handler.release();
        }

        self.component_handler.borrow_mut().0 = handler;

        if !self.component_handler.borrow().0.is_null() {
            let component_handler = self.component_handler.borrow_mut().0 as *mut *mut _;
            let component_handler: ComPtr<dyn IComponentHandler> = ComPtr::new(component_handler);
            component_handler.add_ref();
        }

        kResultTrue
    }
    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void { null_mut() }
}

impl IPluginBase for AGainController {
    unsafe fn initialize(&self, context: *mut c_void) -> tresult {
        if !self.context.borrow().0.is_null() {
            return kResultFalse;
        }

        self.context.borrow_mut().0 = context;

        let mut unit_info = UnitInfo {
            id:              1,
            parent_unit_id:  0,
            name:            [0; 128],
            program_list_id: -1,
        };

        wstrcpy("Unit1", unit_info.name.as_mut_ptr() as *mut i16);
        self.units.borrow_mut().0.push(unit_info);

        let mut gain_parameter = ParameterInfo {
            id: 0,
            title: [0; 128],
            short_title: [0; 128],
            units: [0; 128],
            step_count: 0,
            default_normalized_value: 0.7,
            unit_id: 1,
            flags: kCanAutomate as i32,
        };

        wstrcpy("Gain", gain_parameter.title.as_mut_ptr());
        wstrcpy("Gain", gain_parameter.short_title.as_mut_ptr());
        wstrcpy("%", gain_parameter.units.as_mut_ptr());
        self.parameters.borrow_mut().0.push((gain_parameter, 1.0));

        let mut bypass_parameter = ParameterInfo {
            id: 1,
            title: [0; 128],
            short_title: [0; 128],
            units: [0; 128],
            step_count: 1,
            default_normalized_value: 0.0,
            unit_id: 0,
            flags: kCanAutomate as i32 | kIsBypass as i32,
        };

        wstrcpy("Bypass", bypass_parameter.title.as_mut_ptr());
        self.parameters.borrow_mut().0.push((bypass_parameter, 0.0));

        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        self.units.borrow_mut().0.clear();
        self.parameters.borrow_mut().0.clear();

        if !self.component_handler.borrow().0.is_null() {
            let component_handler = self.component_handler.borrow_mut().0 as *mut *mut _;
            let component_handler: ComPtr<dyn IComponentHandler> = ComPtr::new(component_handler);
            component_handler.release();
            self.component_handler.borrow_mut().0 = null_mut();
        }

        self.context.borrow_mut().0 = null_mut();
        kResultOk
    }
}

impl IUnitInfo for AGainController {
    unsafe fn get_unit_count(&self) -> i32 { 1 }

    unsafe fn get_unit_info(&self, unit_index: i32, info: *mut UnitInfo) -> i32 {
        if unit_index >= 0 && unit_index < self.units.borrow().0.len() as i32 {
            *info = self.units.borrow().0[unit_index as usize];
            return kResultTrue;
        }

        kResultFalse
    }

    unsafe fn get_program_list_count(&self) -> i32 { 0 }

    unsafe fn get_program_list_info(&self, _list_index: i32, _info: *mut ProgramListInfo) -> i32 { kResultFalse }

    unsafe fn get_program_name(&self, _list_id: i32, _program_index: i32, _name: *mut u16) -> i32 { kResultFalse }

    unsafe fn get_program_info(
        &self,
        _list_id: i32,
        _program_index: i32,
        _attribute_id: *const u8,
        _attribute_value: *mut u16,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn has_program_pitch_names(&self, _id: i32, _index: i32) -> i32 { kResultFalse }

    unsafe fn get_program_pitch_name(&self, _id: i32, _index: i32, _pitch: i16, _name: *mut u16) -> i32 { kResultFalse }

    unsafe fn get_selected_unit(&self) -> i32 { 0 }

    unsafe fn select_unit(&self, _id: i32) -> i32 { kResultFalse }

    unsafe fn get_unit_by_bus(&self, _type_: i32, _dir: i32, _index: i32, _channel: i32, _unit_id: *mut i32) -> i32 {
        kResultFalse
    }

    unsafe fn set_unit_program_data(
        &self,
        _list_or_unit: i32,
        _program_index: i32,
        _data: VstPtr<dyn IBStream>,
    ) -> i32 {
        kResultFalse
    }
}

#[no_mangle]
#[allow(non_snake_case)]
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
