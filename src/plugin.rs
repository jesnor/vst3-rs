use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    plugin_parameter::{NormalizedParameterValue, ParameterId, ParameterInfo},
    vst_stream::{VstInStream, VstOutStream},
};

pub trait Plugin {
    fn initialize(&self) -> bool { true }
    fn terminate(&self) -> bool { true }
}

pub trait Parameters {
    fn get_parameters(&self) -> &[&ParameterInfo];
    fn get_normalized_parameter_value(&self, param: &ParameterInfo) -> NormalizedParameterValue;
    fn set_normalized_parameter_value(&self, param: &ParameterInfo, value: NormalizedParameterValue);

    fn get_parameter_by_id(&self, id: ParameterId) -> Option<&ParameterInfo> {
        for p in self.get_parameters() {
            if p.id == id {
                return Some(p);
            }
        }

        None
    }
}

pub trait State {
    fn set_state(&self, stream: &mut VstInStream) -> std::io::Result<()>;
    fn get_state(&self, stream: &mut VstOutStream) -> std::io::Result<()>;
}

pub fn read_parameter_values<T: Parameters>(obj: &T, stream: &mut VstInStream) -> std::io::Result<()> {
    let param_count = stream.read_u32::<LittleEndian>()?;

    // Reset all parameters
    for p in obj.get_parameters() {
        obj.set_normalized_parameter_value(p, p.default_normalized_value)
    }

    // Read parameter values from state
    for _ in 0..param_count {
        let id = stream.read_u32::<LittleEndian>()?;
        let value = stream.read_f64::<LittleEndian>()?;

        if let Some(p) = obj.get_parameter_by_id(id.into()) {
            obj.set_normalized_parameter_value(p, value.into())
        }
    }

    Ok(())
}

pub fn write_parameter_values<T: Parameters>(obj: &T, stream: &mut VstOutStream) -> std::io::Result<()> {
    stream.write_u32::<LittleEndian>(obj.get_parameters().len() as u32)?;

    for p in obj.get_parameters() {
        stream.write_u32::<LittleEndian>(*p.id)?;
        stream.write_f64::<LittleEndian>(*obj.get_normalized_parameter_value(p))?;
    }

    Ok(())
}
