use std::{marker::PhantomData, ops::Deref};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeCell<M, T> {
    value:    T,
    _phantom: PhantomData<M>,
}

impl<M, T> TypeCell<M, T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            _phantom: Default::default(),
        }
    }

    #[inline]
    pub const fn get(&self) -> &T { &self.value }

    #[inline]
    pub fn set(&mut self, value: T) { self.value = value }
}

impl<M, T> Deref for TypeCell<M, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.value }
}

impl<M, T> From<T> for TypeCell<M, T> {
    #[inline]
    fn from(value: T) -> Self { Self::new(value) }
}
