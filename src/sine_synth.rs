use crate::{
    plugin::{AudioProcessor, EditController, Parameter, ParameterValue, Plugin, ProcessInput, ProcessOutput},
    vst_factory::{AudioProcessorInfo, AudioProcessorType, FactoryInfo, VstPluginFactory},
};
use flexi_logger::{DeferredNow, Logger, Record};
use log::info;
use std::{cell::Cell, f64::consts::PI};
use vst3_com::{c_void, sys::GUID};

#[derive(Default, Clone)]
struct SineSynth {
    pos: Cell<f64>,
}

impl SineSynth {
    fn do_process<'t, T>(
        &self,
        input: &'t ProcessInput<'t, T>,
        output: &'t mut ProcessOutput<'t, T>,
        f: impl Fn(f64) -> T,
    ) {
        let s = self.pos.get();
        let c = 440.0 * 2.0 * PI / input.context.sample_rate;

        for bus in output.buses().iter_mut() {
            for channel in bus.channels().iter_mut() {
                channel.is_silenced = false;

                for (i, sample) in channel.samples.iter_mut().enumerate() {
                    let v = 0.5 * (c * (s + i as f64)).sin();
                    *sample = f(v);
                }
            }
        }

        let ns = s + c * input.sample_count as f64;
        self.pos.set(ns.fract());
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
    params: Vec<Parameter>,
}

impl SineSynthController {
    fn new() -> Self {
        Self {
            params: vec![Parameter {
                id: 1,
                title: "Gainer".into(),
                short_title: "Gn".into(),
                units: "%".into(),
                step_count: 0,
                default_normalized_value: 0.0,
                unit_id: 0,

                flags: crate::plugin::ParameterFlags {
                    can_automate:      true,
                    is_read_only:      false,
                    is_wrap_around:    false,
                    is_list:           false,
                    is_program_change: false,
                    is_bypass:         false,
                },

                value_to_string:           Box::new(|value| format!("{} %", (value * 100.0) as i32)),
                string_to_value:           Box::new(|_| None),
                normalized_to_plain_value: Box::new(|value| value),
                plain_to_normalized_value: Box::new(|value| value),
                value:                     Default::default(),
            }],
        }
    }
}

impl Plugin for SineSynthController {}

impl EditController for SineSynthController {
    fn parameters(&self) -> &[Parameter] { &self.params }
    fn get_param_normalized(&self, param: &Parameter) -> ParameterValue { param.value.get() }
    fn set_param_normalized(&self, param: &Parameter, value: ParameterValue) { param.value.set(value) }
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
