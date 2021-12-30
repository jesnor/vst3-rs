use crate::{
    plugin::{AudioProcessor, Plugin, ProcessOutput, ProcessInput},
    vst_factory::{AudioProcessorInfo, AudioProcessorType},
};
use vst3_com::{c_void, sys::GUID};

struct SineSynth {}

impl Plugin for SineSynth {
    fn initialize(&mut self) { todo!() }

    fn terminate(&mut self) { todo!() }
}

impl AudioProcessor for SineSynth {
    fn process<'t, 'u>(&'t mut self, data: &'u ProcessInput<'u>) -> ProcessOutput<'t> {
        Default::default()
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

#[no_mangle]
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    let mut f = Factory::new("My Inc.", "http://www.url.com/test", "sune@sven.se");

    let api = AudioProcessorInfo {
        name:                  "Sine Synth".into(),
        version:               "v0.1.0".into(),
        typ:                   AudioProcessorType::Synth,
        is_distributable:      true,
        simple_mode_supported: false,
    };

    f.add_audio_processor(&PROCESSOR_CID, &api, || Box::new(SineSynth {}));
    f.add_edit_controller(&CONTROLLER_CID, api.name + " Controller", api.version, || Box::new(SineSynth {}));
    Box::into_raw(f) as *mut c_void
}
