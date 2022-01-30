#![allow(unused_variables)]

use crate::edit_controller::EditController;
use crate::plugin_parameter::ParameterInfo;
use crate::utils::{char16_to_string, string_copy_into_16, string_to_fixed_width_16};
use core::slice;
use log::info;
use std::cell::Cell;
use std::ptr::null_mut;
use vst3_com::interfaces::IUnknown;
use vst3_com::{c_void, ComPtr};
use vst3_sys::base::{kInternalError, kInvalidArgument, kResultFalse, kResultOk, kResultTrue};
use vst3_sys::vst::{
    CtrlNumber, IComponentHandler, IEditController, IEditController2, IEditControllerHostEditing, IMidiMapping,
    IUnitInfo, ParamID, ParameterFlags, ProgramListInfo, TChar, UnitInfo,
};
use vst3_sys::VST3;
use vst3_sys::{
    base::{tresult, FIDString, IBStream, IPluginBase},
    utils::VstPtr,
    vst,
};

fn to_vst_parameter_info(info: &ParameterInfo) -> vst::ParameterInfo {
    vst::ParameterInfo {
        id:                       info.id,
        title:                    string_to_fixed_width_16(&info.title),
        short_title:              string_to_fixed_width_16(&info.short_title),
        units:                    string_to_fixed_width_16(&info.units),
        step_count:               info.step_count,
        default_normalized_value: *info.default_normalized_value,
        unit_id:                  info.unit_id,

        flags: if info.flags.can_automate { ParameterFlags::kCanAutomate as i32 } else { 0 } |
            if info.flags.is_read_only { ParameterFlags::kIsReadOnly as i32 } else { 0 } |
            if info.flags.is_wrap_around { ParameterFlags::kIsWrapAround as i32 } else { 0 } |
            if info.flags.is_list { ParameterFlags::kIsList as i32 } else { 0 } |
            if info.flags.is_program_change { ParameterFlags::kIsProgramChange as i32 } else { 0 } |
            if info.flags.is_bypass { ParameterFlags::kIsBypass as i32 } else { 0 },
    }
}

#[VST3(implements(
    IEditController,
    IEditController2,
    IUnitInfo,
    IPluginBase,
    IEditControllerHostEditing,
    IMidiMapping
))]
pub struct VstEditController {
    controller:        Box<dyn EditController>,
    component_handler: Cell<*mut c_void>,
    context:           Cell<*mut c_void>,
}

impl VstEditController {
    pub fn new(controller: Box<dyn EditController>) -> Box<Self> {
        Self::allocate(controller, Cell::new(null_mut()), Cell::new(null_mut()))
    }
}

impl IUnitInfo for VstEditController {
    unsafe fn get_unit_count(&self) -> i32 {
        info!("IUnitInfo::get_unit_count");
        1
    }

    unsafe fn get_unit_info(&self, unit_index: i32, info: *mut UnitInfo) -> i32 {
        info!("IUnitInfo::get_unit_info {}", unit_index);

        if unit_index == 0 {
            let mut i = &mut *info;
            i.id = 1;
            i.parent_unit_id = 0;
            string_copy_into_16("Unit1", &mut i.name);
            i.program_list_id = -1;
            return kResultTrue;
        }

        kResultFalse
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        info!("IUnitInfo::get_program_list_count");
        0
    }

    unsafe fn get_program_list_info(&self, list_index: i32, _info: *mut ProgramListInfo) -> i32 {
        info!("IUnitInfo::get_program_list_info {}", list_index);
        kResultFalse
    }

    unsafe fn get_program_name(&self, list_id: i32, program_index: i32, _name: *mut u16) -> i32 {
        info!("IUnitInfo::get_program_name {} {}", list_id, program_index);
        kResultFalse
    }

    unsafe fn get_program_info(
        &self,
        list_id: i32,
        program_index: i32,
        _attribute_id: *const u8,
        _attribute_value: *mut u16,
    ) -> i32 {
        info!("IUnitInfo::get_program_info {} {}", list_id, program_index);
        kResultFalse
    }

    unsafe fn has_program_pitch_names(&self, id: i32, index: i32) -> i32 {
        info!("IUnitInfo::has_program_pitch_names {} {}", id, index);
        kResultFalse
    }

    unsafe fn get_program_pitch_name(&self, id: i32, index: i32, pitch: i16, _name: *mut u16) -> i32 {
        info!("IUnitInfo::get_program_pitch_name {} {} {}", id, index, pitch);
        kResultFalse
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        info!("IUnitInfo::get_selected_unit");
        1
    }

    unsafe fn select_unit(&self, id: i32) -> i32 {
        info!("IUnitInfo::select_unit {}", id);

        if id == 1 {
            kResultOk
        }
        else {
            kResultFalse
        }
    }

    unsafe fn get_unit_by_bus(&self, type_: i32, dir: i32, index: i32, channel: i32, unit_id: *mut i32) -> i32 {
        info!("IUnitInfo::get_unit_by_bus {} {} {} {}", type_, dir, index, channel);
        *unit_id = 1;
        kResultOk
    }

    unsafe fn set_unit_program_data(&self, list_or_unit: i32, program_index: i32, _data: VstPtr<dyn IBStream>) -> i32 {
        info!("IUnitInfo::set_unit_program_data {} {}", list_or_unit, program_index);
        kResultFalse
    }
}

impl IEditController for VstEditController {
    unsafe fn set_component_state(&self, state: *mut c_void) -> tresult {
        info!("IEditController::set_component_state");

        if state.is_null() {
            return kResultFalse;
        }

        let state = state as *mut *mut _;
        let state: ComPtr<dyn IBStream> = ComPtr::new(state);

        // let mut num_bytes_read = 0;
        // let mut saved_gain = 0.0;
        // let mut saved_bypass = false;
        // let gain_ptr = &mut saved_gain as *mut f64 as *mut c_void;
        // let bypass_ptr = &mut saved_bypass as *mut bool as *mut c_void;

        // state.read(gain_ptr, mem::size_of::<f64>() as i32, &mut num_bytes_read);

        // state.read(bypass_ptr, mem::size_of::<bool>() as i32, &mut num_bytes_read);

        // self.set_param_normalized(0, saved_gain);
        // self.set_param_normalized(1, if saved_bypass { 1.0 } else { 0.0 });

        kResultOk
    }

    unsafe fn set_state(&self, state: *mut c_void) -> tresult {
        info!("IEditController::set_state");
        kResultOk
    }

    unsafe fn get_state(&self, state: *mut c_void) -> tresult {
        info!("IEditController::get_state");
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        info!("IEditController::get_parameter_count");
        self.controller.get_parameters().len() as i32
    }

    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut vst::ParameterInfo) -> tresult {
        info!("IEditController::get_parameter_info {}", param_index);

        if let Some(p) = self.controller.get_parameters().get(param_index as usize) {
            *info = to_vst_parameter_info(p);
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }

    unsafe fn get_param_string_by_value(&self, id: u32, value_normalized: f64, string: *mut TChar) -> tresult {
        info!("IEditController::get_param_string_by_value {} {}", id, value_normalized);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            let s = self.controller.normalized_parameter_value_to_string(p, value_normalized.into());
            string_copy_into_16(&s, slice::from_raw_parts_mut(string, 128));
            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn get_param_value_by_string(&self, id: u32, string: *const TChar, value_normalized: *mut f64) -> tresult {
        info!("IEditController::get_param_value_by_string {}", id);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            if let Some(v) = self
                .controller
                .string_to_normalized_parameter_value(p, &char16_to_string(slice::from_raw_parts(string, 128)))
            {
                *value_normalized = *v;
                return kResultOk;
            }
        }

        kInvalidArgument
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        info!("IEditController::normalized_param_to_plain {} {}", id, value_normalized);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            *p.normalized_to_plain_converter.convert(value_normalized.into())
        }
        else {
            0.0
        }
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        info!("IEditController::plain_param_to_normalized {} {}", id, plain_value);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            *p.normalized_to_plain_converter.convert_inverse(plain_value.into())
        }
        else {
            0.0
        }
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        info!("IEditController::get_param_normalized {}", id);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            *self.controller.get_normalized_parameter_value(p)
        }
        else {
            0.0
        }
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        info!("IEditController::set_param_normalized {} {}", id, value);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            self.controller.set_normalized_parameter_value(p, value.into());
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }

    unsafe fn set_component_handler(&self, handler: *mut c_void) -> tresult {
        info!("IEditController::set_component_handler");

        if self.component_handler.get() == handler {
            return kResultTrue;
        }

        if !self.component_handler.get().is_null() {
            let component_handler = self.component_handler.get() as *mut *mut _;
            let component_handler: ComPtr<dyn IComponentHandler> = ComPtr::new(component_handler);
            component_handler.release();
        }

        self.component_handler.set(handler);

        if !self.component_handler.get().is_null() {
            let component_handler = self.component_handler.get() as *mut *mut _;
            let component_handler: ComPtr<dyn IComponentHandler> = ComPtr::new(component_handler);
            component_handler.add_ref();
        }

        kResultTrue
    }

    unsafe fn create_view(&self, name: FIDString) -> *mut c_void {
        info!("IEditController::create_view");
        //TODO:
        null_mut()
    }
}

impl IEditController2 for VstEditController {
    unsafe fn set_knob_mode(&self, mode: vst3_sys::vst::KnobMode) -> vst3_sys::base::tresult {
        info!("IEditController2::set_knob_mode {}", mode);
        kResultOk
    }

    unsafe fn open_help(&self, only_check: vst3_sys::base::TBool) -> vst3_sys::base::tresult {
        info!("IEditController2::open_help {}", only_check);
        kResultOk
    }

    unsafe fn oepn_about_box(&self, only_check: vst3_sys::base::TBool) -> vst3_sys::base::tresult {
        info!("IEditController2::oepn_about_box {}", only_check);
        kResultOk
    }
}

impl IPluginBase for VstEditController {
    unsafe fn initialize(&self, context: *mut c_void) -> tresult {
        info!("IPluginBase::initialize controller");

        if !self.context.get().is_null() {
            return kResultFalse;
        }

        self.context.set(context);

        if self.controller.initialize() {
            kResultOk
        }
        else {
            kInternalError
        }
    }

    unsafe fn terminate(&self) -> tresult {
        info!("IPluginBase::terminate controller");

        if !self.component_handler.get().is_null() {
            let component_handler = self.component_handler.get() as *mut *mut _;
            let component_handler: ComPtr<dyn IComponentHandler> = ComPtr::new(component_handler);
            component_handler.release();
            self.component_handler.set(null_mut());
        }

        self.context.set(null_mut());

        if self.controller.terminate() {
            kResultOk
        }
        else {
            kInternalError
        }
    }
}

impl IEditControllerHostEditing for VstEditController {
    unsafe fn begin_edit_from_host(&self, id: ParamID) -> tresult {
        info!("IEditControllerHostEditing::begin_edit_from_host {}", id);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            self.controller.begin_edit_from_host(p);
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }

    unsafe fn end_edit_from_host(&self, id: ParamID) -> tresult {
        info!("IEditControllerHostEditing::end_edit_from_host {}", id);

        if let Some(p) = self.controller.get_parameter_by_id(id) {
            self.controller.end_edit_from_host(p);
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }
}

impl IMidiMapping for VstEditController {
    unsafe fn get_midi_controller_assignment(
        &self,
        bus_index: i32,
        channel: i16,
        midi_cc_number: CtrlNumber,
        param_id: *mut ParamID,
    ) -> tresult {
        info!("IMidiMapping::get_midi_controller_assignment {} {} {}", bus_index, channel, midi_cc_number);

        if let Some(id) = self.controller.get_midi_controller_assignment(bus_index, channel, midi_cc_number) {
            *param_id = id;
            kResultTrue
        }
        else {
            kResultFalse
        }
    }
}
