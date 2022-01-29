use std::{cell::Cell, collections::HashMap};

use vst3_sys::vst::{
    ChordEvent, DataEvent, LegacyMidiCCOutEvent, NoteExpressionTextEvent, NoteExpressionValueEvent, NoteOffEvent,
    NoteOnEvent, PolyPressureEvent, ProcessContext, ProcessModes, ScaleEvent, SymbolicSampleSizes,
};

pub trait Plugin {
    fn initialize(&self) -> bool { true }
    fn terminate(&self) -> bool { true }
}

pub type ParameterId = u32;
pub type ParameterValue = f64;

#[derive(Clone, Default)]
pub struct ParameterPoint {
    pub sample_offset: i32,
    pub value:         ParameterValue,
}

pub struct InputParameterChanges<'t> {
    pub parameter_id: ParameterId,
    pub points:       &'t [ParameterPoint],
}

pub struct OutputParameterChanges {
    pub parameter_id: ParameterId,
    pub points:       Vec<ParameterPoint>,
}

#[derive(Copy, Clone)]
pub enum EventData {
    NoteOn(NoteOnEvent),
    NoteOff(NoteOffEvent),
    Data(DataEvent),
    PolyPressure(PolyPressureEvent),
    NoteExpressionValue(NoteExpressionValueEvent),
    NoteExpressionText(NoteExpressionTextEvent),
    Chord(ChordEvent),
    Scale(ScaleEvent),
    LegacyMidiCcOut(LegacyMidiCCOutEvent),
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Event {
    pub bus_index:     i32,
    pub sample_offset: i32,
    pub ppq_position:  f64,
    pub flags:         u16,
    pub event:         EventData,
}

pub struct InChannel<'t, T> {
    pub is_silenced: bool,
    pub samples:     &'t [T],
}

pub struct InBus<'t, T> {
    pub channels: Vec<InChannel<'t, T>>,
}

pub struct OutChannel<'t, T> {
    pub is_silenced: bool,
    pub samples:     &'t mut [T],
}

pub struct OutBus<'t, T> {
    channels: Vec<OutChannel<'t, T>>,
}

impl<'t, T> OutBus<'t, T> {
    pub fn new(channels: Vec<OutChannel<'t, T>>) -> Self { Self { channels } }
    pub fn channels(&mut self) -> &mut [OutChannel<'t, T>] { self.channels.as_mut_slice() }
}

pub struct ProcessInput<'t, T> {
    pub process_mode:  ProcessModes,
    pub sample_size:   SymbolicSampleSizes,
    pub sample_count:  u32,
    pub buses:         Vec<InBus<'t, T>>,
    pub param_changes: HashMap<ParameterId, Vec<ParameterPoint>>,
    pub events:        Vec<Event>,
    pub context:       &'t ProcessContext,
}

pub struct ProcessOutput<'t, T> {
    buses:             Vec<OutBus<'t, T>>,
    pub param_changes: HashMap<ParameterId, Vec<ParameterPoint>>,
    pub events:        Vec<Event>,
}

impl<'t, T> ProcessOutput<'t, T> {
    pub fn new(buses: Vec<OutBus<'t, T>>) -> Self {
        Self {
            buses,
            param_changes: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn buses(&mut self) -> &mut [OutBus<'t, T>] { self.buses.as_mut_slice() }
}

pub trait AudioProcessor: Plugin {
    fn process_f32<'t>(&self, input: &'t ProcessInput<'t, f32>, output: &'t mut ProcessOutput<'t, f32>);
    fn process_f64<'t>(&self, input: &'t ProcessInput<'t, f64>, output: &'t mut ProcessOutput<'t, f64>);
    fn get_tail_samples(&self) -> u32 { 0 }
}

#[derive(Clone, Default)]
pub struct ParameterFlags {
    pub can_automate:      bool,
    pub is_read_only:      bool,
    pub is_wrap_around:    bool,
    pub is_list:           bool,
    pub is_program_change: bool,
    pub is_bypass:         bool,
}

pub struct Parameter {
    pub id: ParameterId,
    pub title: String,
    pub short_title: String,
    pub units: String,
    pub step_count: i32,
    pub default_normalized_value: ParameterValue,
    pub unit_id: i32,
    pub flags: ParameterFlags,
    pub value_to_string: Box<dyn Fn(ParameterValue) -> String>,
    pub string_to_value: Box<dyn Fn(&str) -> Option<ParameterValue>>,
    pub normalized_to_plain_value: Box<dyn Fn(ParameterValue) -> ParameterValue>,
    pub plain_to_normalized_value: Box<dyn Fn(ParameterValue) -> ParameterValue>,
    pub value: Cell<ParameterValue>,
}

#[allow(unused_variables)]
pub trait EditController: Plugin {
    fn parameters(&self) -> &[Parameter];
    fn get_param_normalized(&self, param: &Parameter) -> ParameterValue;
    fn set_param_normalized(&self, param: &Parameter, value: ParameterValue);

    fn begin_edit_from_host(&self, param: &Parameter) {}
    fn end_edit_from_host(&self, param: &Parameter) {}

    fn get_midi_controller_assignment(&self, bus_index: i32, channel: i16, midi_cc_number: i16) -> Option<ParameterId> {
        None
    }
}
