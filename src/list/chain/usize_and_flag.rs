use core::fmt::{self, Formatter, Debug};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct UsizeAndFlag(pub usize);
impl UsizeAndFlag {
    pub const FLAG_MASK: usize = 1;
    pub const SIZE_MASK: usize = !Self::FLAG_MASK;
    pub const fn size(self) -> usize {
        self.0 & Self::SIZE_MASK
    }
    pub const fn flag(self) -> bool {
        (self.0 & Self::FLAG_MASK) != 0
    }
    pub fn set_size(&mut self, size: usize) {
        *self = Self::new(size, self.flag())
    }
    pub fn set_flag(&mut self, flag: bool) {
        *self = Self::new(self.size(), flag)
    }
    pub const fn new(size: usize, flag: bool) -> Self {
        Self((size & Self::SIZE_MASK) | (flag as usize))
    }
}
impl Debug for UsizeAndFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.size())
            .field(&self.flag())
            .finish()
    }
}
