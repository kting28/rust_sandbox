//! Atomic Types for processors without atomic ops 

use core::{cell::Cell};

#[non_exhaustive]
pub enum Ordering {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
    SeqCst,
}
pub struct AtomicUsize { 
    pub(crate) val: Cell<usize>
}
impl AtomicUsize{

    #[inline]
    pub fn store(&self, val: usize, _ordering: Ordering) {
        self.val.set(val)
    }
    #[inline]
    pub fn load(&self, _ordering: Ordering) -> usize {
        self.val.get()
    }
    pub const fn new(val: usize) -> Self {
        AtomicUsize { val: Cell::new(val)  }
    }
}