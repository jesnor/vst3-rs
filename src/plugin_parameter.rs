use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{
    converter::{Converter, IsoConverter},
    range::Range,
    type_cell::TypeCell,
};

pub type ParameterId = u32;
pub type ParameterValue = f64;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Normalized;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Plain;

pub type NormalizedParameterValue = TypeCell<Normalized, ParameterValue>;
pub type PlainParameterValue = TypeCell<Plain, ParameterValue>;
pub type ParameterValueConverter = Box<dyn IsoConverter<NormalizedParameterValue, PlainParameterValue> + Send + Sync>;
pub type ParameterValueRange = Range<PlainParameterValue>;

#[derive(Clone, Default, PartialEq, Debug, Copy)]
pub struct ParameterPoint {
    pub sample_offset: i32,
    pub value:         NormalizedParameterValue,
}

#[derive(Clone, Default)]
pub struct ParameterFlags {
    pub can_automate:      bool,
    pub is_read_only:      bool,
    pub is_wrap_around:    bool,
    pub is_list:           bool,
    pub is_program_change: bool,
    pub is_bypass:         bool,
}

pub struct ParameterInfo {
    pub id:                            ParameterId,
    pub title:                         String,
    pub short_title:                   String,
    pub units:                         String,
    pub step_count:                    i32,
    pub default_normalized_value:      NormalizedParameterValue,
    pub unit_id:                       i32,
    pub flags:                         ParameterFlags,
    pub normalized_to_plain_converter: ParameterValueConverter,
}

impl ParameterInfo {
    pub fn new_linear(
        id: ParameterId,
        title: &str,
        units: &str,
        default_value: ParameterValue,
        value_range: Range<ParameterValue>,
    ) -> Self {
        let normalized_to_plain_converter = Box::new(LinearParameterConverter::new(value_range.to()));

        Self {
            id,
            title: title.into(),
            short_title: title.into(),
            units: units.into(),
            step_count: 0,
            default_normalized_value: normalized_to_plain_converter.convert_inverse(default_value.into()),
            unit_id: 0,
            flags: ParameterFlags {
                can_automate: true,
                ..Default::default()
            },
            normalized_to_plain_converter,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LinearParameterConverter {
    value_range: ParameterValueRange,
    scale:       ParameterValue,
}

impl LinearParameterConverter {
    pub fn new(value_range: ParameterValueRange) -> Self {
        Self {
            value_range,
            scale: 1.0 / (value_range.max().get() - value_range.min().get()),
        }
    }

    pub fn value_range(&self) -> ParameterValueRange { self.value_range }
}

impl Converter<NormalizedParameterValue, PlainParameterValue> for LinearParameterConverter {
    fn convert(&self, value: NormalizedParameterValue) -> PlainParameterValue {
        PlainParameterValue::new(
            (1.0 - value.get()) * self.value_range.min().get() + value.get() * self.value_range.max().get(),
        )
    }
}

impl IsoConverter<NormalizedParameterValue, PlainParameterValue> for LinearParameterConverter {
    fn convert_inverse(&self, value: PlainParameterValue) -> NormalizedParameterValue {
        ((value.get() - self.value_range.min().get()) * self.scale).into()
    }
}

#[derive(Clone)]
pub struct ParameterWithValue {
    pub parameter:        &'static ParameterInfo,
    pub value:            Cell<PlainParameterValue>,
    pub normalized_value: Cell<NormalizedParameterValue>,
}

impl ParameterWithValue {
    pub fn new(parameter: &'static ParameterInfo, value: PlainParameterValue) -> Self {
        Self {
            parameter,
            value: value.into(),
            normalized_value: parameter.normalized_to_plain_converter.convert_inverse(value).into(),
        }
    }

    pub fn new_normalized(parameter: &'static ParameterInfo, value: NormalizedParameterValue) -> Self {
        Self {
            parameter,
            value: parameter.normalized_to_plain_converter.convert(value).into(),
            normalized_value: value.into(),
        }
    }

    pub fn new_default(parameter: &'static ParameterInfo) -> Self {
        Self::new_normalized(parameter, parameter.default_normalized_value)
    }

    pub fn set(&self, value: PlainParameterValue) {
        self.normalized_value.set(self.parameter.normalized_to_plain_converter.convert_inverse(value));
        self.value.set(value);
    }

    pub fn get(&self) -> PlainParameterValue { self.value.get() }

    pub fn set_normalized(&self, value: NormalizedParameterValue) {
        self.normalized_value.set(value);
        self.value.set(self.parameter.normalized_to_plain_converter.convert(value));
    }

    pub fn get_normalized(&self) -> NormalizedParameterValue { self.normalized_value.get() }

    pub fn update(&self, param_changes: &HashMap<ParameterId, Vec<ParameterPoint>>) -> PlainParameterValue {
        if let Some(v) = param_changes.get(&self.parameter.id).map(|v| v.last().map(|p| p.value)).flatten() {
            self.set_normalized(v);
        }

        self.value.get()
    }
}

#[derive(Clone)]
pub struct ParameterValueContainer {
    params:      &'static [&'static ParameterInfo],
    id_to_param: HashMap<ParameterId, Rc<ParameterWithValue>>,
}

impl ParameterValueContainer {
    pub fn new(params: &'static [&'static ParameterInfo]) -> Self {
        Self {
            params,
            id_to_param: params.iter().map(|p| (p.id, Rc::new(ParameterWithValue::new_default(p)))).collect(),
        }
    }

    pub fn get_value(&self, id: ParameterId) -> Option<&Rc<ParameterWithValue>> { self.id_to_param.get(&id) }
    pub fn clone_value(&self, id: ParameterId) -> Rc<ParameterWithValue> { self.get_value(id).unwrap().clone() }
}
