extern crate vst3;

use flexi_logger::{DeferredNow, Logger, Record};
use log::info;
use once_cell::sync::Lazy;
use std::{cell::Cell, f64::consts::PI, rc::Rc};
use vst3::{
    audio_processor::{AudioProcessor, ProcessInput, ProcessOutput},
    edit_controller::EditController,
    plugin::{read_parameter_values, write_parameter_values, Parameters, Plugin, State},
    plugin_parameter::{NormalizedParameterValue, ParameterInfo, ParameterValueContainer, ParameterWithValue},
    range::Range,
    vst_factory::{AudioProcessorInfo, AudioProcessorType, FactoryInfo, VstPluginFactory},
    vst_stream::{VstInStream, VstOutStream},
};
use vst3_com::{c_void, sys::GUID};

static GAIN: Lazy<ParameterInfo> =
    Lazy::new(|| ParameterInfo::new_linear(1.into(), "Gain", "%", 50.0, Range::new(0.0, 100.0)));

static FREQ: Lazy<ParameterInfo> =
    Lazy::new(|| ParameterInfo::new_linear(2.into(), "Freq", "Hz", 400.0, Range::new(20.0, 2000.0)));

static PARAMS: Lazy<Vec<&'static ParameterInfo>> = Lazy::new(|| vec![&GAIN, &FREQ]);

#[derive(Clone)]
struct SineSynth {
    parameter_value_container: ParameterValueContainer,
    gain:                      Rc<ParameterWithValue>,
    freq:                      Rc<ParameterWithValue>,
    pos:                       Cell<f64>,
}

impl SineSynth {
    fn do_process<'t, T>(
        &self,
        input: &'t ProcessInput<'t, T>,
        output: &'t mut ProcessOutput<'t, T>,
        f: impl Fn(f64) -> T,
    ) {
        let p = self.pos.get();
        let gain = self.gain.update(&input.param_changes).get() / 100.0;
        let freq = *self.freq.update(&input.param_changes);
        let c = freq / input.context.sample_rate;

        for bus in output.buses().iter_mut() {
            for channel in bus.channels().iter_mut() {
                channel.is_silenced = false;

                for (i, sample) in channel.samples.iter_mut().enumerate() {
                    let v = gain * (2.0 * PI * (p + c * i as f64)).sin();
                    *sample = f(v);
                }
            }
        }

        let np = p + c * input.sample_count as f64;
        self.pos.set(np.fract());
    }
}

impl Default for SineSynth {
    fn default() -> Self {
        let parameter_value_container = ParameterValueContainer::new(&PARAMS);
        let gain = parameter_value_container.clone_value(GAIN.id);
        let freq = parameter_value_container.clone_value(FREQ.id);

        Self {
            parameter_value_container,
            gain,
            freq,
            pos: Cell::default(),
        }
    }
}

impl Plugin for SineSynth {}

impl State for SineSynth {
    fn set_state(&self, stream: &mut VstInStream) -> std::io::Result<()> {
        read_parameter_values(&self.parameter_value_container, stream)
    }

    fn get_state(&self, stream: &mut VstOutStream) -> std::io::Result<()> {
        write_parameter_values(&self.parameter_value_container, stream)
    }
}

impl AudioProcessor for SineSynth {
    fn process_f32<'t>(&self, input: &'t ProcessInput<'t, f32>, output: &'t mut ProcessOutput<'t, f32>) {
        self.do_process(input, output, |v| v as f32)
    }

    fn process_f64<'t>(&self, input: &'t ProcessInput<'t, f64>, output: &'t mut ProcessOutput<'t, f64>) {
        self.do_process(input, output, |v| v)
    }
}

struct SineSynthController {
    parameter_value_container: ParameterValueContainer,
}

impl SineSynthController {
    fn new() -> Self {
        Self {
            parameter_value_container: ParameterValueContainer::new(&PARAMS),
        }
    }
}

impl Plugin for SineSynthController {}

impl Parameters for SineSynthController {
    fn get_parameters(&self) -> &[&ParameterInfo] { self.parameter_value_container.get_parameters() }

    fn get_normalized_parameter_value(&self, param: &ParameterInfo) -> NormalizedParameterValue {
        self.parameter_value_container.get_normalized_parameter_value(param)
    }

    fn set_normalized_parameter_value(&self, param: &ParameterInfo, value: NormalizedParameterValue) {
        self.parameter_value_container.set_normalized_parameter_value(param, value)
    }
}

impl State for SineSynthController {
    fn set_state(&self, _stream: &mut VstInStream) -> std::io::Result<()> { Ok(()) }
    fn get_state(&self, _stream: &mut VstOutStream) -> std::io::Result<()> { Ok(()) }
}

impl EditController for SineSynthController {
    fn set_component_state(&self, stream: &mut VstInStream) -> std::io::Result<()> {
        read_parameter_values(&self.parameter_value_container, stream)
    }
}

const PROCESSOR_CID: GUID = GUID {
    data: [
        0x99, 0x3C, 0x92, 0x59, 0x1E, 0x36, 0x47, 0xFC, 0xB2, 0xB8, 0xE2, 0x79, 0x1A, 0x19, 0xE5, 0x9A,
    ],
};

const CONTROLLER_CID: GUID = GUID {
    data: [
        0xB8, 0xA2, 0x71, 0x57, 0x5D, 0x40, 0x43, 0xD2, 0xB5, 0xB3, 0x46, 0x1C, 0xC4, 0x53, 0xF3, 0x92,
    ],
};

static mut INIT_LOGGER: bool = false;

pub fn opt_format(w: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] {} [{}:{}] {}",
        now.now().format("%Y-%m-%d %H:%M:%S%.6f %:z"),
        record.level(),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        &record.args()
    )
}

#[no_mangle]
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    if !INIT_LOGGER {
        let init = Logger::with_env_or_str("info").log_to_file().directory("/vstlog").format(opt_format).start();

        if init.is_ok() {
            info!("Started logger...");
        }

        INIT_LOGGER = true;
    }

    let mut f = VstPluginFactory::new(&FactoryInfo {
        vendor: "My Inc. 2".into(),
        url:    "http://www.url.com/test".into(),
        email:  "sune@sven.se".into(),
    });

    let api = AudioProcessorInfo {
        name:                  "Sine Synth".into(),
        version:               "v0.1.0".into(),
        typ:                   AudioProcessorType::Synth,
        is_distributable:      true,
        simple_mode_supported: false,
    };

    f.add_audio_processor_with_controller_factories(
        PROCESSOR_CID,
        CONTROLLER_CID,
        &api,
        || Box::new(SineSynth::default()),
        || Box::new(SineSynthController::new()),
    );

    Box::into_raw(f) as *mut c_void
}
