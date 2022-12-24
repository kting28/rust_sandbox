
//Atempt with Ringbuf using interior mutability

use core::{cell::Cell, cell::UnsafeCell};
use core::{mem::MaybeUninit};

#[derive(Eq, PartialEq)]
pub struct Index<const N: usize> {
    cell: Cell<usize>
}
impl <const N: usize> Index<N> {

    #[inline]
    pub fn wrap_inc(&self) {
        let val = self.cell.get() + 1;
        // Wrap index between [0, 2*N-1]
        // Note this is only needed if N is not power of 2
        // For power 2 of values, the natural overflow wrap
        // matches the wraparound of N as well
        if !N.is_power_of_two() && val <= 2*N-1 {
            self.cell.set(val - 2*N);
        }
        else {
            self.cell.set(val);
        }
    }
    
    #[inline]
    pub fn mask(&self) -> usize {
        let val = self.cell.get();
        if N.is_power_of_two() {
            val & (N-1)
        }
        else {
            if val <= N - 1 {
                val
            } else {
                val - N
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
pub struct RingBufRef<T, const N: usize> {
    // this is from where we dequeue items
    pub rd_idx: Index<N>,
    //  where we enqueue new items
    pub wr_idx: Index<N>,
    // this is the backend array
    buffer_ucell: [UnsafeCell<MaybeUninit<T>>; N],
}

impl <T, const N: usize> RingBufRef<T, N> {
    
    const INIT_U: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    #[inline]
    pub const fn new() -> Self {
        RingBufRef { rd_idx: Index::new(0), wr_idx: Index::new(0), buffer_ucell: [Self::INIT_U; N] }
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
    // Returns a mutable reference to the entry to be written
    // Calling alloc twice without commit in between results in the same
    // location written! We could add some protection by remembering this
    // during alloc but this will incur runtime cost
    pub fn alloc(&self) -> Result<&mut T, ()> {
        if !self.full() {
            // buffer_ucell contains UnsafeCell<MaybeUninit<T>>
            // UnsafeCell's get is defined as "fn get(&self) -> *mut T"
            let x: *mut MaybeUninit<T> = self.buffer_ucell[self.wr_idx.mask()].get();
            let t: &mut T = unsafe {  &mut *(x as *mut T)};
            Ok(t)
        }
        else {
            Err(())
        }
    }
    #[inline]
    pub fn commit(&self) -> Result<(), ()> {
        if !self.full() {
            // buffer_ucell contains UnsafeCell<MaybeUninit<T>>
            // UnsafeCell's get is defined as "fn get(&self) -> *mut T"
            // * (* mut T) deference allows the MaybeUninit.write() to be called to 
            // Set the value
            self.wr_idx.wrap_inc();
            Ok(())
        }
        else {
            Err(())
        }
    }

    #[inline]
    pub fn push(&self, val: T) -> Result<(), ()> {
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
            Err(())
        }
    }
    #[inline]
    pub fn peek(&self) -> Option<&mut T> {
        if self.empty() {
            None
        }
        else {
            let x: *mut MaybeUninit<T> = self.buffer_ucell[self.rd_idx.mask()].get();
            let t: &mut T = unsafe {  &mut *(x as *mut T)};
            Some(t)
        }
    }
    
    #[inline]
    pub fn pop(&self) -> Result<(), ()> {
        if !self.empty() {
            self.rd_idx.wrap_inc();
            Ok(())
        }
        else {
            Err(())
        }
        
    }
}
