
//! Fixed capacity Single Producer Single Consumer Ringbuffer with no mutex protection.
//! Implementation based on https://www.snellman.net/blog/archive/2016-12-13-ring-buffers/
//! This version is for demonstration only. The value T is copied in and out of the array
//! on push/peek

use core::{cell::Cell, cell::UnsafeCell};
use core::{mem::MaybeUninit};

/// Internal Index struct emcapsulating masking and wrapping operations
/// according to size const size N
#[derive(Eq, PartialEq)]
pub struct Index<const N: usize> {
    cell: Cell<usize>
}
impl <const N: usize> Index<N> {

    #[inline]
    pub fn wrap_inc(&self) {

        // Wrapping increment by 1 first
        let val = self.cell.get().wrapping_add(1);

        // Wrap index between [0, 2*N-1]
        // For power 2 of values, the natural overflow wrap
        // matches the wraparound of N. Hence the manual wrap
        // below is not required for power of 2 N
        if !N.is_power_of_two() && val > 2*N-1 {
            // val = val - 2*N
            self.cell.set(val.wrapping_sub(2*N));
        }
        else {
            self.cell.set(val);
        }
    }
    
    // Mask the value for indexing [0, N-1]
    #[inline]
    pub fn mask(&self) -> usize {
        let val = self.cell.get();
        if N.is_power_of_two() {
            val & (N-1)
        }
        else {
            if val > N - 1 {
                val - N
            } else {
                val
            }
        }
    }
    #[inline]
    pub fn get(&self) -> usize {
        self.cell.get()
    }
    pub const fn new(val: usize) -> Self {
        Index { cell: Cell::new(val) }
    }
}

pub struct RingBuf<T, const N: usize> {
    // this is from where we dequeue items
    pub rd_idx: Index<N>,
    //  where we enqueue new items
    pub wr_idx: Index<N>,
    // this is the backend array
    buffer_ucell: [UnsafeCell<MaybeUninit<T>>; N],
}

impl <T: core::marker::Copy, const N: usize> RingBuf<T, N> {
    
    const INIT_U: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    #[inline]
    pub const fn new() -> Self {
        RingBuf { rd_idx: Index::new(0), wr_idx: Index::new(0), buffer_ucell: [Self::INIT_U; N] }
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.rd_idx == self.wr_idx
    }

    #[inline]
    pub fn size(&self) -> usize {
        // wrapping sub
        self.wr_idx.get().wrapping_sub(self.rd_idx.get())
    }
    #[inline]
    pub fn full(&self) -> bool {
        self.size() == N
    }
    #[inline]
    // The Result<> return enforces handling of return type
    // I.e. if user does not check for push success, the compiler
    // generates warnings
    pub fn push(&self, val: T) -> Result<(), T> {
        if !self.full() {
            // buffer_ucell contains UnsafeCell<MaybeUninit<T>>
            // UnsafeCell's get is defined as "fn get(&self) -> *mut T"
            // * (* mut T) deference allows the MaybeUninit.write() to be called to 
            // Set the value
            unsafe {(*self.buffer_ucell[self.wr_idx.mask()].get()).write(val);}
            self.wr_idx.wrap_inc();
            Ok(())
        }
        else {
            Err(val)
        }
    }
    #[inline]
    pub fn peek(&self) -> Option<T> {
        if self.empty() {
            None
        }
        else {
            let val = unsafe {*(self.buffer_ucell[self.rd_idx.mask()].get() as *const T)};
            Some(val)
        }
    }

    #[inline]
    pub fn pop(&self) -> Option<T> {
        let res = self.peek();
        if res.is_some() {
            self.rd_idx.wrap_inc();
        }
        res
    }
}
