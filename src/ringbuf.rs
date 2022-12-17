
//Atempt with Ringbuf using interior mutability

use core::{cell::Cell, cell::UnsafeCell};
use core::{mem::MaybeUninit};

pub struct RingBuf<T, const N: usize> {
    // this is from where we dequeue items
    pub rd_idx: Cell<usize>,
    //  where we enqueue new items
    pub wr_idx: Cell<usize>,
    // this is the backend array
    pub buffer_ucell: [UnsafeCell<MaybeUninit<T>>; N],
}


impl <T: core::marker::Copy, const N: usize> RingBuf<T, N> {
    
    const INIT_U: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    #[inline]
    pub const fn new() -> Self {
        RingBuf { rd_idx: Cell::new(0), wr_idx: Cell::new(0), buffer_ucell: [Self::INIT_U; N] }
    }

    #[inline]
    pub fn wrap(val: usize) -> usize {
        // Wrap index between [0, 2*N-1]
        //TODO: Note this is only needed if N is not power of 2
        // For power 2 of values, the natural overflow wrap
        // matches the wraparound of N as well
        if val <= 2*N - 1 {
            val
        } else {
            val - 2*N
        }
    }

    #[inline]
    pub fn mask(val: usize) -> usize {
        if val <= N - 1 {
            val
        } else {
            val - N
        }
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.rd_idx == self.wr_idx
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.wr_idx.get().wrapping_sub(self.rd_idx.get())
    }
    #[inline]
    pub fn full(&self) -> bool {
        self.size() == N
    }
    #[inline]
    pub fn push(&self, val: T) {
        assert!(!self.full());
        unsafe {(*self.buffer_ucell[Self::mask(self.wr_idx.get())].get()).write(val);}
        self.wr_idx.set(Self::wrap(self.wr_idx.get()+1));
    }
    #[inline]
    pub fn pop(& self) -> T {
        assert!(!self.empty());
        let val = unsafe {*(self.buffer_ucell[Self::mask(self.rd_idx.get())].get() as *const T)};
        self.rd_idx.set(Self::wrap(self.rd_idx.get() + 1));
        val
    }
 
}



