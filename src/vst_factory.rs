#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use crate::plugin::{AudioProcessor, EditController};
use crate::utils::{char8_to_16, string_to_fixed_width};
use crate::vst_audio_processor::VstAudioProcessor;
use crate::vst_edit_controller::VstEditController;
use log::info;
use std::cell::Cell;
use std::ptr::null_mut;
use vst3_com::{c_void, IID};
use vst3_sys::base::{
    kInvalidArgument, kResultFalse, kResultOk, tresult, ClassCardinality, FactoryFlags, IPluginFactory,
    IPluginFactory2, IPluginFactory3, PClassInfo, PClassInfo2, PClassInfoW, PFactoryInfo,
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

#[derive(Clone, Copy)]
pub enum AudioProcessorFlag {
    Distributable = 1,
    SimpleModeSupported = 2,
}

#[derive(Clone, Copy)]
pub enum AudioProcessorType {
    Synth,
}

const VST_VERSION_STRING: &str = "VST 3.6.13";

#[derive(Clone)]
pub struct FactoryInfo {
    pub vendor: String,
    pub url:    String,
    pub email:  String,
}

#[derive(Clone)]
pub struct AudioProcessorInfo {
    pub name:                  String,
    pub version:               String,
    pub typ:                   AudioProcessorType,
    pub is_distributable:      bool,
    pub simple_mode_supported: bool,
}

pub fn new_pclass_info2(
    cid: IID,
    category: &str,
    name: &str,
    flags: u32,
    subcategories: &str,
    vendor: &str,
    version: &str,
) -> PClassInfo2 {
    PClassInfo2 {
        cid,
        cardinality: ClassCardinality::kManyInstances as i32,
        category: string_to_fixed_width(category),
        name: string_to_fixed_width(name),
        class_flags: flags,
        subcategories: string_to_fixed_width(subcategories),
        vendor: string_to_fixed_width(vendor),
        version: string_to_fixed_width(version),
        sdk_version: string_to_fixed_width(VST_VERSION_STRING),
    }
}

#[VST3(implements(IPluginFactory3, IPluginFactory2, IPluginFactory))]
pub struct VstPluginFactory {
    info:    FactoryInfo,
    pinfo:   PFactoryInfo,
    classes: Vec<(PClassInfo2, Box<dyn Fn() -> *mut c_void>)>,
    context: Cell<*mut c_void>,
}

impl VstPluginFactory {
    pub fn new(info: &FactoryInfo) -> Box<Self> {
        Self::allocate(
            info.clone(),
            PFactoryInfo {
                vendor: string_to_fixed_width(&info.vendor),
                url:    string_to_fixed_width(&info.url),
                email:  string_to_fixed_width(&info.email),
                flags:  FactoryFlags::kComponentNonDiscardable as i32,
            },
            Vec::default(),
            Cell::new(null_mut()),
        )
    }

    pub fn add_audio_processor_with_controller_factories(
        &mut self,
        processor_cid: IID,
        controller_cid: IID,
        info: &AudioProcessorInfo,
        processor_factory: impl Fn() -> Box<dyn AudioProcessor> + 'static,
        controller_factory: impl Fn() -> Box<dyn EditController> + 'static,
    ) {
        self.add_edit_controller(
            controller_cid,
            &(info.name.clone() + " Controller"),
            &info.version,
            controller_factory,
        );

        self.add_audio_processor_factory(processor_cid, controller_cid, info, processor_factory);
    }

    pub fn add_audio_processor_factory(
        &mut self,
        cid: IID,
        controller_cid: IID,
        info: &AudioProcessorInfo,
        factory: impl Fn() -> Box<dyn AudioProcessor> + 'static,
    ) {
        let fb = Box::new(factory);

        let f = move || {
            info!("Creating audio processor");
            let vap = VstAudioProcessor::new(controller_cid, fb());
            Box::into_raw(vap) as *mut c_void
        };

        self.add_class_factory(
            cid,
            "Audio Module Class",
            &info.name,
            if info.is_distributable { AudioProcessorFlag::Distributable as u32 } else { 0 } |
                if info.simple_mode_supported { AudioProcessorFlag::SimpleModeSupported as u32 } else { 0 },
            match info.typ {
                AudioProcessorType::Synth => "Instrument|Synth",
            },
            &info.version,
            f,
        )
    }

    pub fn add_edit_controller(
        &mut self,
        cid: IID,
        name: &str,
        version: &str,
        factory: impl Fn() -> Box<dyn EditController> + 'static,
    ) {
        let fb = Box::new(factory);

        let f = move || {
            info!("Creating edit controller");
            let vec = VstEditController::new(fb());
            Box::into_raw(vec) as *mut c_void
        };

        self.add_class_factory(cid, "Component Controller Class", name, 0, "", version, f)
    }

    pub fn add_class_factory(
        &mut self,
        cid: IID,
        category: &str,
        name: &str,
        flags: u32,
        subcategories: &str,
        version: &str,
        factory: impl Fn() -> *mut c_void + 'static,
    ) {
        let info = new_pclass_info2(cid, category, name, flags, subcategories, &self.info.vendor, version);
        self.classes.push((info, Box::new(factory)));
    }
}

unsafe fn copy<T>(src: *const T, dst: *mut T) { std::ptr::copy_nonoverlapping(src, dst, 1); }

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
        self.context.set(context);
        kResultOk
    }
}

impl IPluginFactory2 for VstPluginFactory {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        if let Some((ci, _)) = self.classes.get(index as usize) {
            copy(ci, info);
            kResultOk
        }
        else {
            kInvalidArgument
        }
    }
}

impl IPluginFactory for VstPluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        info!("IPluginFactory.get_factory_info");
        copy(&self.pinfo, info);
        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        info!("IPluginFactory.count_classes");
        self.classes.len() as i32
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> tresult {
        info!("IPluginFactory.get_class_info");

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
        info!("IPluginFactory.create_instance");

        for (ci, f) in self.classes.iter() {
            if ci.cid == *cid {
                info!("IPluginFactory.create_instance found class");
                *obj = f();
                return kResultOk;
            }
        }

        info!("IPluginFactory.create_instance end");
        kResultFalse
    }
}
