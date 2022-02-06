use crate::audio_processor::{
    AudioProcessor, Event, EventData, InBus, InChannel, OutBus, OutChannel, ProcessInput, ProcessOutput,
};
use crate::plugin_parameter::{ParameterId, ParameterPoint};
use crate::utils::string_copy_into_i16;
use crate::vst_stream::{VstInStream, VstOutStream};
use core::slice;
use log::info;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ptr::null_mut;
use uuid::Uuid;
use vst3_com::{c_void, ComPtr, IID};
use vst3_sys::base::{kInvalidArgument, kNotImplemented, kResultTrue, IBStream, TBool};
use vst3_sys::vst::{
    BusDirection, BusInfo, BusType, IEventList, IParamValueQueue, IParameterChanges, IoMode, MediaType, ProcessModes,
    ProcessSetup, RoutingInfo, SpeakerArrangement, SymbolicSampleSizes,
};
use vst3_sys::VST3;
use vst3_sys::{
    base::{kResultFalse, kResultOk, tresult, IPluginBase},
    vst::{IAudioProcessor, IComponent, K_SAMPLE32, K_SAMPLE64},
};

unsafe fn to_plugin_event(e: &vst3_sys::vst::Event) -> Option<Event> {
    let t = match e.type_ {
        0 => Some(EventData::NoteOn(e.event.note_on)),
        1 => Some(EventData::NoteOff(e.event.note_off)),
        2 => Some(EventData::Data(e.event.data)),
        3 => Some(EventData::PolyPressure(e.event.poly_pressure)),
        4 => Some(EventData::NoteExpressionValue(e.event.note_expression_value)),
        5 => Some(EventData::NoteExpressionText(e.event.note_expression_text)),
        6 => Some(EventData::Chord(e.event.chord)),
        7 => Some(EventData::Scale(e.event.scale)),
        65535 => Some(EventData::LegacyMidiCcOut(e.event.legacy_midi_cc_out)),
        _ => None,
    };

    t.map(|event| Event {
        bus_index: e.bus_index,
        sample_offset: e.sample_offset,
        ppq_position: e.ppq_position,
        flags: e.flags,
        event,
    })
}

pub struct AudioBus {
    name:        String,
    bus_type:    BusType,
    flags:       i32,
    active:      bool,
    speaker_arr: SpeakerArrangement,
}

fn get_channel_count(arr: SpeakerArrangement) -> i32 {
    let mut arr = arr;
    let mut count = 0;
    while arr != 0 {
        if (arr & 1) == 1 {
            count += 1;
        }
        arr >>= 1;
    }
    count
}

#[VST3(implements(IComponent, IAudioProcessor, IPluginBase))]
pub struct VstAudioProcessor {
    controller_cid:       Uuid,
    processor:            Box<dyn AudioProcessor>,
    current_process_mode: Cell<i32>,
    process_setup:        Cell<ProcessSetup>,
    audio_inputs:         RefCell<Vec<AudioBus>>,
    audio_outputs:        RefCell<Vec<AudioBus>>,
    gain:                 Cell<f64>,
    bypass:               Cell<bool>,
    context:              Cell<*mut c_void>,
}

impl VstAudioProcessor {
    pub fn new(controller_cid: Uuid, processor: Box<dyn AudioProcessor>) -> Box<Self> {
        Self::allocate(
            controller_cid,
            processor,
            Cell::default(),
            Cell::default(),
            RefCell::default(),
            RefCell::default(),
            Cell::default(),
            Cell::default(),
            Cell::new(null_mut()),
        )
    }

    pub unsafe fn setup_processing_ae(&self, new_setup: *const ProcessSetup) -> tresult {
        if self.can_process_sample_size((*new_setup).symbolic_sample_size) != kResultTrue {
            return kResultFalse;
        }

        self.process_setup.set(*new_setup);
        kResultOk
    }

    pub fn add_audio_input(&self, name: &str, arr: SpeakerArrangement) {
        let new_bus = AudioBus {
            name:        name.into(),
            bus_type:    0,
            flags:       1,
            active:      false,
            speaker_arr: arr,
        };

        self.audio_inputs.borrow_mut().push(new_bus);
    }

    pub fn add_audio_output(&self, name: &str, arr: SpeakerArrangement) {
        let new_bus = AudioBus {
            name:        name.into(),
            bus_type:    0,
            flags:       1,
            active:      false,
            speaker_arr: arr,
        };

        self.audio_outputs.borrow_mut().push(new_bus);
    }
}

impl IComponent for VstAudioProcessor {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> tresult {
        info!("IComponent::get_controller_class_id");
        (*tuid).data = *self.controller_cid.as_bytes();
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: IoMode) -> tresult {
        info!("IComponent::set_io_mode");
        kNotImplemented
    }

    unsafe fn get_bus_count(&self, type_: MediaType, dir: BusDirection) -> i32 {
        info!("IComponent::get_bus_count");

        match type_ {
            0 => match dir {
                0 => self.audio_inputs.borrow().len() as i32,
                _ => self.audio_outputs.borrow().len() as i32,
            },

            _ => 0,
        }
    }

    unsafe fn get_bus_info(&self, type_: MediaType, dir: BusDirection, index: i32, info: *mut BusInfo) -> tresult {
        info!("IComponent::get_bus_info");
        (*info).media_type = type_;
        (*info).direction = dir;

        match type_ {
            0 => {
                let buses = if dir == 0 { &self.audio_inputs } else { &self.audio_outputs };

                if let Some(bus) = buses.borrow().get(index as usize) {
                    string_copy_into_i16(&bus.name, &mut (*info).name);
                    (*info).channel_count = get_channel_count(bus.speaker_arr);
                    (*info).bus_type = bus.bus_type;
                    (*info).flags = bus.flags as u32;
                    kResultTrue
                }
                else {
                    kInvalidArgument
                }
            }

            _ => kResultFalse,
        }
    }

    unsafe fn get_routing_info(&self, _in_info: *mut RoutingInfo, _out_info: *mut RoutingInfo) -> tresult {
        info!("IComponent::get_routing_info");
        kNotImplemented
    }

    unsafe fn activate_bus(&self, type_: MediaType, dir: BusDirection, index: i32, state: TBool) -> tresult {
        info!("IComponent::activate_bus");

        match type_ {
            0 => {
                let buses = if dir == 0 { &self.audio_inputs } else { &self.audio_outputs };

                if let Some(bus) = buses.borrow_mut().get_mut(index as usize) {
                    bus.active = state != 0;
                    kResultTrue
                }
                else {
                    kInvalidArgument
                }
            }

            _ => kInvalidArgument,
        }
    }

    unsafe fn set_active(&self, _state: TBool) -> tresult {
        info!("IComponent::set_active");
        kResultOk
    }

    unsafe fn set_state(&self, state: *mut c_void) -> tresult {
        info!("IComponent::set_state");

        if state.is_null() {
            return kResultFalse;
        }

        let stream: ComPtr<dyn IBStream> = ComPtr::new(state as *mut *mut _);

        if self.processor.set_state(&mut VstInStream::new(stream)).is_ok() {
            kResultOk
        }
        else {
            kResultFalse
        }
    }

    unsafe fn get_state(&self, state: *mut c_void) -> tresult {
        info!("IComponent::get_state");

        if state.is_null() {
            return kResultFalse;
        }

        let stream: ComPtr<dyn IBStream> = ComPtr::new(state as *mut *mut _);

        if self.processor.get_state(&mut VstOutStream::new(stream)).is_ok() {
            kResultOk
        }
        else {
            kResultFalse
        }
    }
}

impl IPluginBase for VstAudioProcessor {
    unsafe fn initialize(&self, context: *mut c_void) -> tresult {
        info!("IPluginBase::initialize audio");

        if !self.context.get().is_null() {
            return kResultFalse;
        }

        self.context.set(context);
        self.add_audio_input("Stereo In", 3);
        self.add_audio_output("Stereo Out", 3);
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        info!("IPluginBase::terminate audio");
        self.audio_inputs.borrow_mut().clear();
        self.audio_outputs.borrow_mut().clear();
        self.context.set(null_mut());
        kResultOk
    }
}

impl IAudioProcessor for VstAudioProcessor {
    unsafe fn set_bus_arrangements(
        &self,
        _inputs: *mut SpeakerArrangement,
        _num_ins: i32,
        _outputs: *mut SpeakerArrangement,
        _num_outs: i32,
    ) -> tresult {
        info!("IAudioProcessor::set_bus_arrangements");
        kResultFalse
    }

    unsafe fn get_bus_arrangement(&self, dir: BusDirection, index: i32, arr: *mut SpeakerArrangement) -> tresult {
        info!("IAudioProcessor::get_bus_arrangement");
        let buses = if dir == 0 { &self.audio_inputs } else { &self.audio_outputs }.borrow();

        if let Some(bus) = buses.get(index as usize) {
            *arr = bus.speaker_arr;
            kResultTrue
        }
        else {
            kResultFalse
        }
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> tresult {
        match symbolic_sample_size {
            K_SAMPLE32 | K_SAMPLE64 => kResultTrue,
            _ => kResultFalse,
        }
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        info!("IAudioProcessor::get_latency_samples");
        0
    }

    unsafe fn setup_processing(&self, setup: *const ProcessSetup) -> tresult {
        info!("IAudioProcessor::setup_processing");
        self.current_process_mode.set((*setup).process_mode);
        self.setup_processing_ae(setup)
    }

    unsafe fn set_processing(&self, _state: TBool) -> tresult {
        info!("IAudioProcessor::set_processing");
        kNotImplemented
    }

    unsafe fn process(&self, data: *mut vst3_sys::vst::ProcessData) -> tresult {
        unsafe fn create_data<'t, T>(
            data: &vst3_sys::vst::ProcessData,
        ) -> Option<(ProcessInput<'t, T>, ProcessOutput<'t, T>)> {
            if data.num_inputs == 0 && data.num_outputs == 0 {
                return None;
            }

            let process_mode = match data.process_mode {
                0 => ProcessModes::kOffline,
                1 => ProcessModes::kPrefetch,
                2 => ProcessModes::kRealtime,
                _ => return None,
            };

            let sample_size = match data.symbolic_sample_size {
                0 => SymbolicSampleSizes::kSample32,
                1 => SymbolicSampleSizes::kSample64,
                _ => return None,
            };

            let mut param_changes: HashMap<ParameterId, Vec<ParameterPoint>> = HashMap::new();

            if let Some(ipc) = data.input_param_changes.upgrade() {
                let param_count = ipc.get_parameter_count();
                param_changes.reserve(param_count as usize);

                // Convert input parameter changes
                for i in 0..ipc.get_parameter_count() {
                    let param_queue = ipc.get_parameter_data(i);

                    if let Some(param_queue) = param_queue.upgrade() {
                        let point_count = param_queue.get_point_count();

                        let v = param_changes
                            .entry(param_queue.get_parameter_id().into())
                            .or_insert_with(|| Vec::with_capacity(point_count as usize));

                        for j in 0..point_count {
                            let mut value = 0.0;
                            let mut sample_offset = 0;

                            if param_queue.get_point(j, &mut sample_offset as *mut _, &mut value as *mut _) != kResultOk
                            {
                                return None;
                            }

                            v.push(ParameterPoint {
                                sample_offset,
                                value: value.into(),
                            });
                        }
                    }
                    else {
                        return None;
                    }
                }
            }

            let mut events = Vec::new();

            if let Some(ie) = data.input_events.upgrade() {
                let ec = ie.get_event_count();
                events.reserve(ec as usize);

                // Convert input events
                for i in 0..ec {
                    let mut e = vst3_sys::vst::Event {
                        bus_index:     0,
                        sample_offset: 0,
                        ppq_position:  0.0,
                        flags:         0,
                        type_:         0,
                        event:         vst3_sys::vst::EventData {
                            note_on: vst3_sys::vst::NoteOnEvent {
                                channel:  0,
                                pitch:    0,
                                tuning:   0f32,
                                velocity: 0f32,
                                length:   0,
                                note_id:  0,
                            },
                        },
                    };

                    if ie.get_event(i, &mut e as *mut _) != kResultOk {
                        return None;
                    }

                    match to_plugin_event(&e) {
                        Some(event) => events.push(event),
                        _ => return None,
                    }
                }
            }

            let mut input_buses: Vec<InBus<'t, T>> = Vec::new();
            let mut output_buses: Vec<OutBus<'t, T>> = Vec::new();

            for bus in slice::from_raw_parts(data.inputs, data.num_inputs as usize) {
                let channels = (0..bus.num_channels)
                    .map(|ci| {
                        let b = bus.buffers.offset(ci as isize);

                        InChannel::<'t, T> {
                            is_silenced: (bus.silence_flags >> ci) == 1,
                            samples:     slice::from_raw_parts(b as *const T, data.num_samples as usize),
                        }
                    })
                    .collect::<Vec<InChannel<'t, T>>>();

                input_buses.push(InBus { channels });
            }

            for bus in slice::from_raw_parts(data.outputs, data.num_outputs as usize) {
                let channels = slice::from_raw_parts_mut(bus.buffers as *mut *mut T, bus.num_channels as usize)
                    .iter()
                    .map(|ch| OutChannel {
                        is_silenced: false,
                        samples:     slice::from_raw_parts_mut(*ch, data.num_samples as usize),
                    })
                    .collect();

                output_buses.push(OutBus::new(channels));
            }

            let input = ProcessInput {
                process_mode,
                sample_size,
                sample_count: data.num_samples as u32,
                buses: input_buses,
                param_changes,
                events,
                context: &*data.context,
            };

            let output = ProcessOutput::new(output_buses);
            Some((input, output))
        }

        let data = &*data;

        if data.symbolic_sample_size == K_SAMPLE32 {
            if let Some((i, mut o)) = create_data(data) {
                self.processor.process_f32(&i, &mut o);
                return kResultOk;
            }
        }
        else if let Some((i, mut o)) = create_data(data) {
            self.processor.process_f64(&i, &mut o);
            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn get_tail_samples(&self) -> u32 { self.processor.get_tail_samples() }
}
