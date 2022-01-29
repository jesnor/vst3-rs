use crate::{
    plugin::{
        AudioProcessor, EditController, Parameter, ParameterId, ParameterPoint, ParameterValue, Plugin, ProcessInput,
        ProcessOutput,
    },
    vst_factory::{AudioProcessorInfo, AudioProcessorType, FactoryInfo, VstPluginFactory},
};
use flexi_logger::{DeferredNow, Logger, Record};
use log::info;
use std::{cell::Cell, collections::HashMap, f64::consts::PI};
use vst3_com::{c_void, sys::GUID};

lazy_static! {
    static ref GAIN: Parameter = Parameter::new_linear(1, "Gain", "%", 50.0, 0.0, 100.0);
    static ref FREQ: Parameter = Parameter::new_linear(2, "Freq", "Hz", 400.0, 20.0, 2000.0);
    static ref PARAMS: Vec<&'static Parameter> = vec![&GAIN, &FREQ];
}

#[derive(Clone)]
struct ParameterWithValue {
    param: &'static Parameter,
    value: Cell<ParameterValue>,
}

impl ParameterWithValue {
    fn new(param: &'static Parameter) -> Self {
        Self {
            param,
            value: param.default_normalized_value.into(),
        }
    }

    fn update(&self, param_changes: &HashMap<ParameterId, Vec<ParameterPoint>>) -> ParameterValue {
        if let Some(v) = param_changes.get(&self.param.id).map(|v| v.last().map(|p| p.value)).flatten() {
            self.value.set(v);
        }

        (self.param.normalized_to_plain_value)(self.value.get())
    }
}

#[derive(Clone)]
struct SineSynth {
    pos:  Cell<f64>,
    gain: ParameterWithValue,
    freq: ParameterWithValue,
}

impl SineSynth {
    fn do_process<'t, T>(
        &self,
        input: &'t ProcessInput<'t, T>,
        output: &'t mut ProcessOutput<'t, T>,
        f: impl Fn(f64) -> T,
    ) {
        let p = self.pos.get();
        let gain = self.gain.update(&input.param_changes) / 100.0;
        let freq = self.freq.update(&input.param_changes);
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
        Self {
            pos:  Cell::default(),
            gain: ParameterWithValue::new(&GAIN),
            freq: ParameterWithValue::new(&FREQ),
        }
    }
}

impl Plugin for SineSynth {}

impl AudioProcessor for SineSynth {
    fn process_f32<'t>(&self, input: &'t ProcessInput<'t, f32>, output: &'t mut ProcessOutput<'t, f32>) {
        self.do_process(input, output, |v| v as f32)
    }

    fn process_f64<'t>(&self, input: &'t ProcessInput<'t, f64>, output: &'t mut ProcessOutput<'t, f64>) {
        self.do_process(input, output, |v| v)
    }
}

struct SineSynthController {
    param_values: HashMap<ParameterId, Cell<f64>>,
}

impl SineSynthController {
    fn new() -> Self {
        Self {
            param_values: PARAMS.iter().map(|p| (p.id, p.default_normalized_value.into())).collect(),
        }
    }
}

impl Plugin for SineSynthController {}

impl EditController for SineSynthController {
    fn parameters(&self) -> &[&Parameter] { &PARAMS }

    fn get_param_normalized(&self, param: &Parameter) -> ParameterValue {
        self.param_values.get(&param.id).map(|c| c.get()).unwrap_or_else(|| param.default_normalized_value)
    }

    fn set_param_normalized(&self, param: &Parameter, value: ParameterValue) {
        if let Some(c) = self.param_values.get(&param.id) {
            c.set(value)
        }
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
