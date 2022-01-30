use std::marker::PhantomData;

pub trait Converter<From, To> {
    fn convert(&self, value: From) -> To;
}

pub trait IsoConverter<From, To>: Converter<From, To> {
    fn convert_inverse(&self, value: To) -> From;
}

struct InverseIsoConverter<From, To, T: IsoConverter<From, To>> {
    pub converter: T,
    _phantom1:     PhantomData<From>,
    _phantom2:     PhantomData<To>,
}

impl<From, To, T: IsoConverter<From, To>> InverseIsoConverter<From, To, T> {
    pub fn new(converter: T) -> Self {
        Self {
            converter,
            _phantom1: Default::default(),
            _phantom2: Default::default(),
        }
    }

    pub fn invert(&self) -> &T { &self.converter }
}
