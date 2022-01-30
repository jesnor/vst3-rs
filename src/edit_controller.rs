use crate::{
    plugin::Plugin,
    plugin_parameter::{NormalizedParameterValue, ParameterId, ParameterInfo, ParameterValue},
};

#[allow(unused_variables)]
pub trait EditController: Plugin {
    fn get_parameters(&self) -> &[&ParameterInfo];
    fn get_normalized_parameter_value(&self, param: &ParameterInfo) -> NormalizedParameterValue;
    fn set_normalized_parameter_value(&self, param: &ParameterInfo, value: NormalizedParameterValue);

    fn normalized_parameter_value_to_string(&self, param: &ParameterInfo, value: NormalizedParameterValue) -> String {
        format!("{:.1} {}", param.normalized_to_plain_converter.convert(value).get(), param.units)
    }

    fn string_to_normalized_parameter_value(
        &self,
        param: &ParameterInfo,
        value: &str,
    ) -> Option<NormalizedParameterValue> {
        value.parse::<ParameterValue>().ok().map(|v| v.into())
    }

    /// Should be overridden for performance reasons when there are many parameters
    fn get_parameter_by_id(&self, id: ParameterId) -> Option<&ParameterInfo> {
        for p in self.get_parameters() {
            if p.id == id {
                return Some(p);
            }
        }

        None
    }

    fn begin_edit_from_host(&self, param: &ParameterInfo) {}
    fn end_edit_from_host(&self, param: &ParameterInfo) {}

    fn get_midi_controller_assignment(&self, bus_index: i32, channel: i16, midi_cc_number: i16) -> Option<ParameterId> {
        None
    }
}
