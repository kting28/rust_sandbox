/*
Embedded Rust Examples
*/
//allows definition of panic, eh_personality
#![feature(lang_items, core_intrinsics)] 
// no rust std library
#![no_std] 
//define our own main, otherwise fn main depends of std
#![no_main] 

pub mod spsc;

#[macro_use]
extern crate bitfield;
extern crate libc_print;
use libc_print::std_name::{println};

use spsc::Queue;

// Structure Examples
#[derive(Copy, Clone)]
struct Sub {
    // Structure with array of 4 integers
    arr: [i32; 4],
}

// Structure with references to arrays
// Notice the explicity life time needed
struct Parent<'a> {
    id: i8,
    data: &'a [i32],    // reference
    data1: &'a mut Sub, // mutable reference
}

// Unions
#[repr(C)]
// Unions must implement Copy, since this union contains
// Sub, Sub must implement Copy
#[derive(Copy, Clone)]
union UnionP {
    id: i32,
    data: i32,
    data1: [i32; 4],
    sub: Sub,
}

// bitfield crate
// TODO: does BitField1 require repr(C) ?
bitfield!{
  #[derive(Copy, Clone)]
  // No more than 32 bits
  pub struct BfStruct(u32);
  //impl Debug;
  u32; // this is optional
  // The fields default to u32
  pub bf1, set_bf1: 8, 0;
  pub bf2, set_bf2 : 31, 9;
  pub all, set_all : 31, 0;
}


// Bitfield with u8 array backing
bitfield! {
  #[derive(Copy, Clone)]
  pub struct BfStructByteArr([u8]);
  impl Debug;
  // Default type of following fields
  // Must be less than the width of the 
  // field defined below
  u8;
  // The fields default to u32
  pub bf1, set_bf1: 7, 0;
  pub bf2, set_bf2 : 14, 8;
  pub bf3, set_bf3 : 16, 15;
  pub bf4, set_bf4 : 17, 17;
  // Type of the "all" field below
  u32;
  pub all, set_all : 17, 0;
}


// Structure with bitfield
#[repr(C)]
#[derive(Copy, Clone)]
struct Struct1 {
    id: i32,
    bf_arry : [BfStruct; 4]
}

// Helpers for this example
fn print_type_of<T>(_: &T) {
    println!("{}", core::any::type_name::<T>())
}

#[no_mangle]
//fn main()
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // The explit types are not required but is encouraged
    // Initial Sub with input array of u32
    let mut y: Sub = Sub { arr: [1, 2, 3, 4] };
    // Array of structures
    // Since UnionP implements Copy, 4 items can be initialized as such
    const ARR_U: [UnionP; 4] = [ UnionP {id:7}; 4 ];
    let x: Parent = Parent {
        id: 0,
        data: &[1, 2, 3],
        data1: &mut y, // Sub is borrowed here
    };

    //let x1: Parent = Parent {
    //    id: 0,
    //    data: &[1, 2, 3],
    //    data1: &mut y, // Sub is borrowed 2nd time here , won't compile
    //};

    println!("x.id={}, x.data[2]={:?}, x.data1.arr[2]={:?}", x.id, x.data[2], x.data1.arr[2]);

    //y.arr[1] = 1; // Cannot do this here since y is still mutably borrowed!

    // This is the last use of x
    println!("x.id={}", x.id);

    y.arr[1] = 1;

    // Union access needs to wrapped in unsafe
    unsafe {
        println!("ARR_U id={}", ARR_U[1].id);
    }

    // Crate bitfield example
    let bitfield1: BfStruct = BfStruct(0x554455);
    // field1 = 0x55
    // field2 = 0x554455>>9
    println!("bf1={:#x}, bf2={:#x} all>>9={:#x}", bitfield1.bf1(), bitfield1.bf2(), bitfield1.all());

    //Actual type BitField2<[u8; 3]> is derived
    let bitfield2 = BfStructByteArr([0xcc, 0xaa, 0x55]);
    println!("bf1={:#x}, bf2={:#x}, bf3={:#x} (all>>15)&3={:#x}", 
        bitfield2.bf1(), 
        bitfield2.bf2(), 
        bitfield2.bf3(), 
        (bitfield2.all()>>15)&3);

    println!("size of BitField1 {}", core::mem::size_of::<BfStruct>());
    println!("size of BitField2 {}", core::mem::size_of::<BfStructByteArr<[u8;3]>>());
    print_type_of(&bitfield2);

    // Crate spsc::Queue
    let mut rb: Queue<u8, 4> = Queue::new();
    assert!(rb.enqueue(0).is_ok());
    assert!(rb.enqueue(1).is_ok());
    assert!(rb.enqueue(2).is_ok());
    assert!(rb.enqueue(3).is_err()); // full
    assert_eq!(rb.dequeue(), Some(0));

    return 0;

}

// These functions are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
    //core::intrinsics::abort()
}


// The first of these functions, rust_eh_personality, is used by the failure mechanisms of the
// compiler. This is often mapped to GCC's personality function (see the libstd implementation for
// more information), but crates which do not trigger a panic can be assured that this function is
// never called. The language item's name is eh_personality. #[lang = "eh_personality"]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}