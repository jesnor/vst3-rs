use crate::utils::{char8_to_16, to_fixed_width_str};
use std::cell::RefCell;
use std::ptr::null_mut;
use vst3_com::{c_void, IID};
use vst3_sys::base::{
    kInvalidArgument, kResultFalse, kResultOk, tresult, FactoryFlags, IPluginFactory, IPluginFactory2, IPluginFactory3,
    PClassInfo, PClassInfo2, PClassInfoW, PFactoryInfo,
};
use vst3_sys::VST3;

fn class_info2_to_w(i: &PClassInfo2) -> PClassInfoW {
    PClassInfoW {
        cid:           i.cid,
        cardinality:   i.cardinality,
        category:      i.category,
        name:          char8_to_16(&i.name),
        class_flags:   i.class_flags,
        subcategories: i.subcategories,
        vendor:        char8_to_16(&i.vendor),
        version:       char8_to_16(&i.version),
        sdk_version:   char8_to_16(&i.sdk_version),
    }
}

enum AudioProcessorFlag {
    Distributable = 1,
    SimpleModeSupported = 2,
}

pub enum AudioProcessorType {
    Synth,
}

const VST_VERSION_STRING: &str = "VST 3.6.13";

struct FactoryInfo {
    pub vendor: String,
    pub url:    String,
    pub email:  String,
}

pub struct AudioProcessorInfo {
    pub name:                  String,
    pub version:               String,
    pub typ:                   AudioProcessorType,
    pub is_distributable:      bool,
    pub simple_mode_supported: bool,
}

#[VST3(implements(IPluginFactory3, IPluginFactory2, IPluginFactory))]
pub struct VstPluginFactory {
    info:    PFactoryInfo,
    classes: Vec<(PClassInfo2, Box<dyn Fn() -> *mut c_void>)>,
    context: RefCell<*mut c_void>,
}

/*pub fn create_audio_processor(
    audio_processor_cid: &IID,
    controller_cid: &IID,
    factory_info: &FactoryInfo,
    info: &AudioProcessorInfo,
    factory: impl AudioProcessorFactory + 'static,
) -> *mut c_void {
    let ai = PClassInfo2 {
        cid: *audio_processor_cid,
        cardinality: ClassCardinality::kManyInstances as i32,
        category: to_fixed_width_str("Audio Module Class"),
        name: to_fixed_width_str(&info.name),
        class_flags: if info.is_distributable {
            AudioProcessorFlag::Distributable as u32
        } else {
            0
        },
        subcategories: to_fixed_width_str(match info.typ {
            AudioProcessorType::Synth => "Instrument|Synth",
        }),
        vendor: to_fixed_width_str(&factory_info.vendor),
        version: to_fixed_width_str(&info.version),
        sdk_version: to_fixed_width_str(VST_VERSION_STRING),
    };

    let

    let ci = PClassInfo2 {
        cid: *controller_cid,
        cardinality: ClassCardinality::kManyInstances as i32,
        category: to_fixed_width_str("Component Controller Class"),
        name: to_fixed_width_str(&(info.name + " Controller")),
        class_flags: 0,
        subcategories: to_fixed_width_str(""),
        vendor: to_fixed_width_str(&factory_info.vendor),
        version: to_fixed_width_str(&info.version),
        sdk_version: to_fixed_width_str(VST_VERSION_STRING),
    };


    VstPluginFactory::new(factory_info, vec![(ci, Box)

    ]
    [
        ,
        ,
    ]
}*/

impl VstPluginFactory {
    pub fn new(factory_info: &FactoryInfo, classes: &Vec<(PClassInfo2, Box<dyn Fn() -> *mut c_void>)>) -> *mut c_void {
        let f = Self::allocate(
            PFactoryInfo {
                vendor: to_fixed_width_str(&factory_info.vendor),
                url:    to_fixed_width_str(&factory_info.url),
                email:  to_fixed_width_str(&factory_info.email),
                flags:  FactoryFlags::kComponentNonDiscardable as i32,
            },
            *classes,
            RefCell::new(null_mut()),
        );

        Box::into_raw(f) as *mut c_void
    }
}

impl IPluginFactory3 for VstPluginFactory {
    unsafe fn get_class_info_unicode(&self, index: i32, info: *mut PClassInfoW) -> tresult {
        if let Some((ci, _)) = self.classes.get(index as usize) {
            *info = class_info2_to_w(ci);
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }

    unsafe fn set_host_context(&self, context: *mut c_void) -> tresult {
        *self.context.borrow_mut() = context;
        kResultOk
    }
}

impl IPluginFactory2 for VstPluginFactory {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        if let Some((ci, _)) = self.classes.get(index as usize) {
            *info = *ci;
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }
}

impl IPluginFactory for VstPluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        *info = self.info;
        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 { self.classes.len() as i32 }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> tresult {
        if let Some((ci, _)) = self.classes.get(index as usize) {
            let info = &mut *info;
            info.cardinality = ci.cardinality;
            info.cid = ci.cid;
            info.category = ci.category;
            info.name = ci.name;
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }

    unsafe fn create_instance(&self, cid: *const IID, _iid: *const IID, obj: *mut *mut c_void) -> tresult {
        for (ci, f) in self.classes {
            if ci.cid == *cid {
                *obj = f();
                return kResultOk;
            }
        }

        kResultFalse
    }
}
