use std::collections::HashMap;

use vst3_sys::vst::{
    ChordEvent, DataEvent, LegacyMidiCCOutEvent, NoteExpressionTextEvent, NoteExpressionValueEvent, NoteOffEvent,
    NoteOnEvent, PolyPressureEvent, ProcessContext, ProcessModes, ScaleEvent, SymbolicSampleSizes,
};

use crate::{
    plugin::Plugin,
    plugin_parameter::{NormalizedParameterValue, ParameterId, ParameterPoint},
};

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

impl<'t, T> ProcessInput<'t, T> {
    pub fn get_last_param_value(&self, id: ParameterId) -> Option<NormalizedParameterValue> {
        self.param_changes.get(&id).map(|v| v.last().map(|p| p.value)).flatten()
    }
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
