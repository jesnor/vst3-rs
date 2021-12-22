#[derive(Debug)]
pub enum Fx {
    /// Scope, FFT-Display, Loudness Processing...
    Analyzer,

    /// Delay, Multi-tap Delay, Ping-Pong Delay...
    Delay,

    /// Amp Simulator, Sub-Harmonic, SoftClipper...
    Distortion,

    /// Compressor, Expander, Gate, Limiter, Maximizer, Tape Simulator, EnvelopeShaper...
    Dynamics,

    ///Equalization, Graphical EQ...
    EQ,

    ///WahWah, ToneBooster, Specific Filter,...
    Filter,

    /// Fx which could be loaded as Instrument too
    Instrument,

    /// Fx which could be loaded as Instrument too and is external (wrapped Hardware)
    InstrumentExternal,

    /// MonoToStereo, StereoEnhancer,...
    Spatial,

    /// Tone Generator, Noise Generator...
    Generator,

    /// Dither, Noise Shaping,...
    Mastering,

    /// Phaser, Flanger, Chorus, Tremolo, Vibrato, AutoPan, Rotary, Cloner...
    Modulation,

    /// Pitch Processing, Pitch Correction, Vocal Tuning...
    PitchShift,

    /// Denoiser, Declicker,...
    Restoration,

    /// Reverberation, Room Simulation, Convolution Reverb...
    Reverb,

    /// dedicated to surround processing: LFE Splitter, Bass Manager...
    Surround,

    /// Volume, Mixer, Tuner...
    Tools,

    /// using Network
    Network,

    /// others type (not categorized)
    Other,
}

#[derive(Debug)]
pub enum Instrument {
    /// Instrument for Drum sounds
    Drum,

    /// External Instrument (wrapped Hardware)
    External,

    /// Instrument for Piano sounds
    Piano,

    /// Instrument based on Samples
    Sampler,

    /// Instrument based on Synthesis
    Synth,

    /// Instrument based on Synthesis and Samples
    SynthSampler,

    /// Effect used as instrument (sound generator), not as insert
    Other,
}

#[derive(Debug)]
pub enum AudioProcessorCategory {
    Fx(Fx),
    Instrument(Instrument),

    /// used for SurroundPanner
    Spatial,

    /// used for SurroundPanner and as insert effect
    SpatialFx,

    /// indicates that it supports only realtime process call, no processing faster than realtime
    OnlyRealTime,

    /// used for plug-in offline processing  (will not work as normal insert plug-in)
    OnlyOfflineProcess,

    /// used for plug-ins that require ARA to operate (will not work as normal insert plug-in)
    OnlyARA,

    /// will be NOT used for plug-in offline processing (will work as normal insert plug-in)
    NoOfflineProcess,

    /// used for Mixconverter/Up-Mixer/Down-Mixer
    UpDownMix,

    /// Meter, Scope, FFT-Display, not selectable as insert plug-in
    Analyzer,

    /// used for Ambisonics channel (FX or Panner/Mixconverter/Up-Mixer/Down-Mixer when combined with other category)
    Ambisonics,

    /// used for Mono only plug-in [optional]
    Mono,

    /// used for Stereo only plug-in [optional]
    Stereo,

    /// used for Surround only plug-in [optional]
    Surround,
}

pub fn to_vst_category_string(cat: &AudioProcessorCategory) -> String {
    match cat {
        AudioProcessorCategory::Fx(fx) => {
            "Fx".to_owned()
                + &match fx {
                    Fx::Other => "".to_owned(),
                    _ => format!("|{:?}", fx),
                }
        }

        AudioProcessorCategory::Instrument(instr) => {
            "Instrument".to_owned()
                + &match instr {
                    Instrument::Other => "".to_owned(),
                    Instrument::SynthSampler => "|Synth|Sampler".to_owned(),
                    _ => format!("|{:?}", instr),
                }
        }

        AudioProcessorCategory::SpatialFx => "Spatial|Fx".to_owned(),
        AudioProcessorCategory::OnlyRealTime => "OnlyRT".to_owned(),
        AudioProcessorCategory::UpDownMix => "Up-Downmix".to_owned(),
        _ => format!("|{:?}", cat),
    }
}
