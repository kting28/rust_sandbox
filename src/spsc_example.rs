use crate::RingBufRef;
use crate::SharedSingleton;

use libc_print::std_name::println;

bitfield!{
    //#[derive(Copy, Clone)]
    // No more than 32 bits
    pub struct SysTime(u32);
    //impl Debug;
    u32; // this is optional
    // The fields default to u32
    pub unit_slot, set_bf1: 2, 0;
    pub slot, set_slot: 9, 3;
    pub sfn, set_sfn : 31, 10;
    pub all, set_all : 31, 0;
}

// Define initialization value to avoid deriving Copy/Clone
// When initializing
pub const SYS_TIME_0: SysTime = SysTime(0);

enum CommandType {
    Process,
}

bitfield!{
    //#[derive(Copy, Clone)]
    // No more than 32 bits
    pub struct Header(u32);
    //impl Debug;
    u32; // this is optional
    // The fields default to u32
    pub cmd_type, set_cmd_type: 2, 0;
    pub cfg_idx, set_cfg_idx: 3, 3;
}
pub const HEADER_0: Header = Header(0);

pub struct Command {
    pub header: Header,
    pub sys_time: SysTime,
}

struct SubCfg {
    // Structure with array of 4 integers
    sub_cfg_arr: [i32; 4],
}

pub struct Cfg {
    id: u32,
    sub_cfg: SubCfg,
}

// A 4-deep ring buffer
const CMD_Q_DEPTH: usize = 4;
// 2 associated payload location (some commands don't use payloads)
const CMD_PAYLOAD_DEPTH: usize = 2;
pub struct Interface {
    cmd_q: RingBufRef<Command, CMD_Q_DEPTH>,
    payload: [SharedSingleton<Cfg>; CMD_PAYLOAD_DEPTH]
}

// Set up 4 entries of interfaces
const NUM_INTFS: usize = 4;
// Init value, suppress the clippy warning for declaring const interior mutable "A “non-constant”
// const item is a legacy way to supply an initialized value to downstream"
#[allow(clippy::declare_interior_mutable_const)]
const INTF_INIT: Interface = Interface {cmd_q: RingBufRef::INIT_0, payload: [SharedSingleton::INIT_0; CMD_PAYLOAD_DEPTH]};

// Final instantiation as global
static SHARED_INTF: [Interface; NUM_INTFS] =  [INTF_INIT; NUM_INTFS];


pub fn producer_irq(idx: usize) {

    // This is an internal state used by producers only
    struct ProducerState{
        iter: u32,
        last_cfg_idx: u32,
    }
    
    // Retrieve my persistent state from the static mut scoped to this function
    // only
    let state: &'static mut ProducerState = {
        const INIT_S: ProducerState = ProducerState { iter: 0, last_cfg_idx: 0};
        // Statically allocated once
        static mut S: [ProducerState; NUM_INTFS] = [INIT_S; NUM_INTFS];
        // return the mutable reference of this interface
        unsafe { &mut S[idx] }
    };
    
    // Static reference to the global shared interface
    // Note this is not mutable hence all mutations (writes) go through
    // the defined functions of the interface structure
    let intf: &'static Interface = &SHARED_INTF[idx];

    let alloc_res = intf.cmd_q.alloc();

    if let Ok(cmd) = alloc_res {

        // claim the opposite payload from last time
        let new_idx = state.last_cfg_idx ^ 1;
        if intf.payload[new_idx as usize].is_consumer_owned() {
            println!("p{} No valid command payload, skip producing command this wakeup!", idx);
        }
        else {
            // Set command header and time information
            cmd.header.set_cmd_type(CommandType::Process as u32);
            cmd.sys_time.set_slot(state.iter);
            cmd.sys_time.set_sfn(state.iter >> 7);

            // Claim payload and attach to command
            state.last_cfg_idx = new_idx;
            cmd.header.set_cfg_idx(new_idx);
            let singleton: &SharedSingleton<Cfg> = &intf.payload[new_idx as usize];

            // unwraps panics if get_mut_ref returns None. ownership already checked
            // above
            let payload_ref = singleton.get_mut_ref().unwrap();
            // Set some random data
            payload_ref.id = state.iter+1;
            payload_ref.sub_cfg.sub_cfg_arr[0] = state.iter as i32;
            state.iter += 1;

            // Set the payload owner
            singleton.pass_to_consumer().unwrap();

            // Commit the command
            intf.cmd_q.commit().unwrap();

            println!("p{} iter {} Sent 1 command", state.iter, idx);
        }
    }
    else {
        println!("p{} Command queue full, skip producing command this wakeup!", idx);
    }
    // Increment the state
    state.iter += 1;
}

pub fn consumer_irq(idx: usize) {
    
    // Retrieve my interface
    let intf: &'static Interface = &SHARED_INTF[idx];
    
    while !intf.cmd_q.is_empty() {
        let cmd = intf.cmd_q.peek();
        match cmd {
            Some(cmd) => {
                println!("c{} Received command type {}", idx, cmd.header.cmd_type());
                assert!(intf.payload[cmd.header.cfg_idx() as usize].is_consumer_owned());

                // cmd is not mutable since peek() returns a const reference
                //cmd.sys_time.set_all(0);
                
                // Return the payload
                intf.payload[cmd.header.cfg_idx() as usize].return_to_producer().unwrap();

                // Pop the command
                intf.cmd_q.pop().unwrap();


                println!("c{} Consumed 1 command", idx);
            },
            None => {
                panic!("command queue empty but nothing returned from peek!");
            }
        }
    }
}
