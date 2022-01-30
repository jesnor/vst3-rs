use crate::{
    plugin::Parameters,
    plugin::{Plugin, State},
    plugin_parameter::{NormalizedParameterValue, ParameterId, ParameterInfo, ParameterValue},
    type_cell::TypeCell,
    vst_stream::VstInStream,
};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitIdTag;

pub type UnitId = TypeCell<UnitIdTag, i32>;

pub struct UnitInfo {
    pub id:              UnitId,
    pub parent_unit_id:  UnitId,
    pub name:            String,
    pub program_list_id: ProgramListId,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramListIdTag;

pub type ProgramListId = TypeCell<ProgramListIdTag, i32>;

pub struct ProgramListInfo {
    pub id:            ProgramListId,
    pub name:          String,
    pub program_count: i32,
}

pub enum KnobMode {
    Circular,
    RelativeCircular,
    Linear,
}

pub enum BusDirection {
    Input,
    Output,
}

pub enum MediaType {
    Audio,
    Event,
}

#[allow(unused_variables)]
pub trait EditController: Plugin + Parameters + State {
    // IEditController methods
    fn set_component_state(&self, stream: &mut VstInStream) -> std::io::Result<()>;

    fn normalized_parameter_value_to_string(&self, param: &ParameterInfo, value: NormalizedParameterValue) -> String {
        format!("{:.1} {}", param.normalized_to_plain_converter.convert(value).get(), param.units)
    }

    fn string_to_normalized_parameter_value(
        &self,
        param: &ParameterInfo,
        value: &str,
    ) -> Option<NormalizedParameterValue> {
        value.parse::<ParameterValue>().ok().map(|v| v.into())
    }

    // IEditController2 methods

    fn set_knob_mode(&self, mode: KnobMode) -> bool { false }
    fn is_open_help_supported(&self) -> bool { false }
    fn open_help(&self) {}
    fn is_open_about_box_supported(&self) -> bool { false }
    fn open_about_box(&self) {}

    // IUnitInfo methods, if you only have one unit these don't have to be overridden

    fn get_units(&self) -> Option<&[UnitInfo]> { None }
    fn get_selected_unit(&self) -> UnitId { 0.into() }
    fn select_unit(&self, id: UnitId) -> bool { false }
    fn get_unit_by_bus(&self, type_: MediaType, dir: BusDirection, index: i32, channel: i32) -> Option<UnitId> { None }

    /// Should be overridden for performance reasons when there are many program lists
    fn get_program_list_by_id(&self, id: ProgramListId) -> Option<&ProgramListInfo> {
        if let Some(pl) = self.get_program_lists() {
            for p in pl {
                if p.id == id {
                    return Some(p);
                }
            }
        }

        None
    }

    fn get_program_lists(&self) -> Option<&[ProgramListInfo]> { None }
    fn get_program_name(&self, program_list: &ProgramListInfo, program_index: i32) -> &str { "" }

    fn get_program_info(&self, program_list: &ProgramListInfo, program_index: i32, attribute_id: &str) -> Option<&str> {
        None
    }

    fn has_program_pitch_names(&self, program_list: &ProgramListInfo, index: i32) -> bool { false }
    fn get_program_pitch_name(&self, program_list: &ProgramListInfo, index: i32, pitch: i16) -> &str { "" }

    // IEditControllerHostEditing methods

    fn begin_edit_from_host(&self, param: &ParameterInfo) {}
    fn end_edit_from_host(&self, param: &ParameterInfo) {}

    // IMidiMapping methods

    fn get_midi_controller_assignment(&self, bus_index: i32, channel: i16, midi_cc_number: i16) -> Option<ParameterId> {
        None
    }
}
