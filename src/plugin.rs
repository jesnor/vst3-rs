use vst3_sys::vst::{
    ChordEvent, DataEvent, LegacyMidiCCOutEvent, NoteExpressionTextEvent, NoteExpressionValueEvent, NoteOffEvent,
    NoteOnEvent, PolyPressureEvent, ProcessContext, ProcessModes, ScaleEvent,
};

pub trait PluginFactory {
    fn create_controller(&self) -> Box<dyn EditController>;
}

pub trait AudioProcessorFactory: PluginFactory {
    fn create_audio_processor(&self) -> Box<dyn AudioProcessor>;
}

pub trait Plugin {
    fn initialize(&mut self);
    fn terminate(&mut self);
}

pub type ParameterId = u32;
pub type ParameterValue = f64;

pub struct ParameterPoint {
    sample_offset: i32,
    value:         ParameterValue,
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
    pub type_:         u16,
    pub event:         EventData,
}

enum Buffers<'t> {
    F32(&'t [&'t [f32]], &'t [&'t mut [f32]]),
    F64(&'t [&'t [f64]], &'t [&'t mut [f64]]),
}

pub struct ProcessInput<'t> {
    pub process_mode:  ProcessModes,
    pub buffers:       Buffers<'t>,
    pub param_changes: &'t [InputParameterChanges<'t>],
    pub events:        &'t [Event],
    pub context:       &'t ProcessContext,
}

#[derive(Default)]
pub struct ProcessOutput<'t> {
    pub output_param_changes: Option<&'t [OutputParameterChanges]>,
    pub output_events:        Option<&'t [Event]>,
}

pub trait AudioProcessor: Plugin {
    fn process<'t, 'u>(&'t mut self, data: &'u ProcessInput<'u>) -> ProcessOutput<'t>;
}

pub trait EditController: Plugin {}
