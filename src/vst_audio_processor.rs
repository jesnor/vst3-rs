use crate::plugin::AudioProcessor;
use crate::utils::wstrcpy;
use std::cell::{Cell, RefCell};
use std::intrinsics::{copy_nonoverlapping, write_bytes};
use std::mem;
use std::ptr::null_mut;
use vst3_com::{c_void, ComPtr, IID};
use vst3_sys::base::{kInvalidArgument, kNotImplemented, kResultTrue, IBStream, TBool};
use vst3_sys::vst::{
    BusDirection, BusInfo, BusType, IEventList, IParamValueQueue, IParameterChanges, IoMode, MediaType, ProcessData,
    ProcessSetup, RoutingInfo, SpeakerArrangement,
};
use vst3_sys::VST3;
use vst3_sys::{
    base::{kResultFalse, kResultOk, tresult, IPluginBase},
    vst::{IAudioProcessor, IComponent, K_SAMPLE32, K_SAMPLE64},
};

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
struct VstAudioProcessor {
    controller_cid:       IID,
    processor:            Box<dyn AudioProcessor>,
    current_process_mode: Cell<i32>,
    process_setup:        RefCell<ProcessSetup>,
    audio_inputs:         RefCell<Vec<AudioBus>>,
    audio_outputs:        RefCell<Vec<AudioBus>>,
    gain:                 Cell<f64>,
    bypass:               Cell<bool>,
    context:              RefCell<*mut c_void>,
}

impl VstAudioProcessor {
    pub unsafe fn setup_processing_ae(&self, new_setup: *const ProcessSetup) -> tresult {
        if self.can_process_sample_size((*new_setup).symbolic_sample_size) != kResultTrue {
            return kResultFalse;
        }

        *self.process_setup.borrow_mut() = (*new_setup).clone();
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
        *tuid = self.controller_cid;
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: IoMode) -> tresult { kNotImplemented }

    unsafe fn get_bus_count(&self, type_: MediaType, dir: BusDirection) -> i32 {
        match type_ {
            0 => match dir {
                0 => self.audio_inputs.borrow().len() as i32,
                _ => self.audio_outputs.borrow().len() as i32,
            },

            _ => 0,
        }
    }

    unsafe fn get_bus_info(&self, type_: MediaType, dir: BusDirection, index: i32, info: *mut BusInfo) -> tresult {
        (*info).media_type = type_;
        (*info).direction = dir;

        match type_ {
            0 => {
                let buses = if dir == 0 { self.audio_inputs } else { self.audio_outputs };

                if let Some(bus) = buses.borrow().get(index as usize) {
                    wstrcpy(&bus.name, (*info).name.as_mut_ptr());
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
        kNotImplemented
    }

    unsafe fn activate_bus(&self, type_: MediaType, dir: BusDirection, index: i32, state: TBool) -> tresult {
        match type_ {
            0 => {
                let buses = if dir == 0 { self.audio_inputs } else { self.audio_outputs };

                if let Some(bus) = buses.borrow().get(index as usize) {
                    buses.borrow_mut()[index as usize].active = state != 0;
                    kResultTrue
                }
                else {
                    kInvalidArgument
                }
            }

            _ => kInvalidArgument,
        }
    }

    unsafe fn set_active(&self, _state: TBool) -> tresult { kResultOk }

    unsafe fn set_state(&self, state: *mut c_void) -> tresult {
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

        self.gain.set(saved_gain);
        self.bypass.set(saved_bypass);

        kResultOk
    }

    unsafe fn get_state(&self, state: *mut c_void) -> tresult {
        if state.is_null() {
            return kResultFalse;
        }

        let state = state as *mut *mut _;
        let state: ComPtr<dyn IBStream> = ComPtr::new(state);

        let mut num_bytes_written = 0;
        let mut gain = self.gain.get();
        let gain_ptr = &mut gain as *mut f64 as *mut c_void;
        let mut bypass = self.bypass.get();
        let bypass_ptr = &mut bypass as *mut bool as *mut c_void;

        state.write(gain_ptr, mem::size_of::<f64>() as i32, &mut num_bytes_written);

        state.write(bypass_ptr, mem::size_of::<bool>() as i32, &mut num_bytes_written);

        kResultOk
    }
}

impl IPluginBase for VstAudioProcessor {
    unsafe fn initialize(&self, context: *mut c_void) -> tresult {
        if !self.context.borrow().is_null() {
            return kResultFalse;
        }

        *self.context.borrow_mut() = context;
        self.add_audio_input("Stereo In", 3);
        self.add_audio_output("Stereo Out", 3);
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        self.audio_inputs.borrow_mut().clear();
        self.audio_outputs.borrow_mut().clear();
        *self.context.borrow_mut() = null_mut();
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
        kResultFalse
    }
    unsafe fn get_bus_arrangement(&self, dir: BusDirection, index: i32, arr: *mut SpeakerArrangement) -> tresult {
        let buses = if dir == 0 { self.audio_inputs } else { self.audio_outputs }.borrow();

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

    unsafe fn get_latency_samples(&self) -> u32 { 0 }

    unsafe fn setup_processing(&self, setup: *const ProcessSetup) -> tresult {
        self.current_process_mode.set((*setup).process_mode);
        self.setup_processing_ae(setup)
    }

    unsafe fn set_processing(&self, _state: TBool) -> tresult { kNotImplemented }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let param_changes = &(*data).input_param_changes;

        if let Some(param_changes) = param_changes.upgrade() {
            let num_params_changed = param_changes.get_parameter_count();

            for i in 0..num_params_changed {
                let param_queue = param_changes.get_parameter_data(i);

                if let Some(param_queue) = param_queue.upgrade() {
                    let mut value = 0.0;
                    let mut sample_offset = 0;
                    let num_points = param_queue.get_point_count();

                    match param_queue.get_parameter_id() {
                        0 => {
                            if param_queue.get_point(num_points - 1, &mut sample_offset as *mut _, &mut value as *mut _) ==
                                kResultTrue
                            {
                                self.gain.set(value);
                            }
                        }

                        1 => {
                            if param_queue.get_point(num_points - 1, &mut sample_offset as *mut _, &mut value as *mut _) ==
                                kResultTrue
                            {
                                self.bypass.set(value > 0.5);
                            }
                        }

                        _ => (),
                    }
                }
            }
        }

        if let Some(input_events) = (*data).input_events.upgrade() {
            let num_events = input_events.get_event_count();
        }

        if (*data).num_inputs == 0 && (*data).num_outputs == 0 {
            return kResultOk;
        }

        let num_channels = (*(*data).inputs).num_channels;
        let num_samples = (*data).num_samples;
        let in_ = (*(*data).inputs).buffers;
        let out_ = (*(*data).outputs).buffers;

        let sample_frames_size = {
            match self.process_setup.borrow().symbolic_sample_size {
                K_SAMPLE32 => (*data).num_samples as usize * mem::size_of::<f32>(),
                K_SAMPLE64 => (*data).num_samples as usize * mem::size_of::<f64>(),
                _ => unreachable!(),
            }
        };

        if (*(*data).inputs).silence_flags != 0 {
            (*(*data).outputs).silence_flags = (*(*data).inputs).silence_flags;
        
            for i in 0..num_channels as isize {
                write_bytes(*out_.offset(i), 0, sample_frames_size);
            }
            
            return kResultOk;
        }

        (*(*data).outputs).silence_flags = 0;

        if self.bypass.get() {
            for i in 0..num_channels as isize {
                if *in_.offset(i) != *out_.offset(i) {
                    copy_nonoverlapping(*in_.offset(i) as *const c_void, *out_.offset(i), sample_frames_size);
                }
            }
        }
        else {
            match self.process_setup.borrow().symbolic_sample_size {
                K_SAMPLE32 => {
                    for i in 0..num_channels as isize {
                        let channel_in = *in_.offset(i) as *const f32;
                        let channel_out = *out_.offset(i) as *mut f32;

                        for j in 0..num_samples as isize {
                            *channel_out.offset(j) = *channel_in.offset(j) * self.gain.get() as f32;
                        }
                    }
                }

                K_SAMPLE64 => {
                    for i in 0..num_channels as isize {
                        let channel_in = *in_.offset(i) as *const f64;
                        let channel_out = *out_.offset(i) as *mut f64;

                        for j in 0..num_samples as isize {
                            *channel_out.offset(j) = *channel_in.offset(j) * self.gain.get();
                        }
                    }
                }

                _ => unreachable!(),
            }
        }

        kResultOk
    }

    unsafe fn get_tail_samples(&self) -> u32 { 0 }
}
