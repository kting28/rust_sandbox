
#![allow(dead_code)]
use core::{cell::Cell, cell::UnsafeCell};
use core::{mem::MaybeUninit};

#[derive(Copy, Clone, PartialEq)]
enum Owner {
    PRODUCER,
    CONSUMER,
}
struct StaticShared <T> {
    owner: Cell<Owner>,
    ucell: UnsafeCell<MaybeUninit<T>>,
}

impl <T> StaticShared<T> {


    const INIT_U: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    #[inline]
    pub const fn new() -> Self {
        StaticShared { owner: Cell::new(Owner::PRODUCER), ucell: Self::INIT_U  }
    }


    #[inline]
    pub fn alloc(&self) -> Option<&mut T> {
        if self.owner.get() == Owner::PRODUCER {
            let x: *mut MaybeUninit<T> = self.ucell.get();
            let t: &mut T = unsafe {  &mut *(x as *mut T)};
            Some(t)
        }
        else {
            None
        }
    }

    /// Pass ownership to CONSUMER only if Owner is already
    /// PRODUCER
    #[inline]
    pub fn commit(&self) -> Result<(),()> {
        if self.owner.get() == Owner::PRODUCER {
            self.owner.set(Owner::CONSUMER);
            Ok(())
        }
        else {
            Err(())
        }
    }

    /// Returns &T is location is owned by CONSUMER
    /// otherwise None
    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.owner.get() == Owner::CONSUMER {
            let x: *mut MaybeUninit<T> = self.ucell.get();
            let t: & T = unsafe {  & *(x as * const T)};
            Some(t)
        }
        else {
            None
        }
    }

    /// Release location back to Producer
    #[inline]
    pub fn release(&self) -> Result<(),()> {
        if self.owner.get() == Owner::CONSUMER {
            self.owner.set(Owner::PRODUCER);
            Ok(())
        }
        else {
            Err(())
        }
    }
}