use std::ops::{Bound, RangeBounds};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Range<T> {
    min: T,
    max: T,
}

impl<T> Range<T> {
    #[inline]
    pub const fn new(min: T, max: T) -> Self { Self { min, max } }

    #[inline]
    pub const fn min(&self) -> &T { &self.min }

    #[inline]
    pub const fn max(&self) -> &T { &self.max }
}

impl<T: Copy> Range<T> {
    #[inline]
    pub fn to<U: From<T> + Copy>(&self) -> Range<U> { Range::new(self.min.into(), self.max.into()) }
}

impl<T: PartialOrd> Range<T> {
    #[inline]
    pub fn contains(&self, v: &T) -> bool { *v >= self.min && *v <= self.max }
}
impl<T: PartialOrd + Copy> Range<T> {
    #[inline]
    pub fn clamp(&self, v: &T) -> T {
        if *v < self.min {
            self.min
        }
        else if *v > self.max {
            self.max
        }
        else {
            *v
        }
    }

    #[inline]
    pub fn extend_with(&self, v: &T) -> Self {
        if *v < self.min {
            Self::new(*v, self.max)
        }
        else if *v > self.max {
            Self::new(self.min, *v)
        }
        else {
            *self
        }
    }
}

impl<T> RangeBounds<T> for Range<T> {
    #[inline]
    fn start_bound(&self) -> std::ops::Bound<&T> { Bound::Included(&self.min) }

    #[inline]
    fn end_bound(&self) -> std::ops::Bound<&T> { Bound::Included(&self.max) }
}
