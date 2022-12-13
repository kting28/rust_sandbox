
pub struct RingBufSimple<T, const N: usize> {
    // this is from where we dequeue items
    pub rd_idx: usize,
    //  where we enqueue new items
    pub wr_idx: usize,
    // this is the backend array
    pub buffer: [T; N],
}


impl <T: core::marker::Copy, const N: usize> RingBufSimple<T, N> {
    
    #[inline]
    pub const fn new(init: T) -> Self {
        RingBufSimple { rd_idx: 0, wr_idx:0, buffer: [init; N ] }
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
        // simple sub will panic on overflow
        self.wr_idx.wrapping_sub(self.rd_idx)
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.size() == N
    }
    #[inline]
    // Must be &mut self, preventing any producer and consumer type of split usage.
    pub fn push(&mut self, val: T) {
        assert!(!self.full());
        self.buffer[Self::mask(self.wr_idx)] = val;
        self.wr_idx = Self::wrap(self.wr_idx+1);
    }
    #[inline]
    // Must be &mut self, preventing any producer and consumer type of split usage.
    pub fn pop(&mut self) -> T {
        assert!(!self.empty());
        let val = self.buffer[Self::mask(self.rd_idx)];
        self.rd_idx = Self::wrap(self.rd_idx + 1);
        val
    }
}
