use crate::RingBufRef;
use crate::SharedSingleton;

use libc_print::std_name::{println};

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
pub const SYS_TIME_0: SysTime = SysTime(0);

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

enum CommandType {
    PROCESS,
    SKIP,
}
pub struct Command {
    pub header: Header,
    pub sys_time: SysTime,
}

struct SubCfg {
    // Structure with array of 4 integers
    arr: [i32; 4],
}

pub struct Cfg {
    id: u32,
    sub_cfg: SubCfg,
}

pub struct Interface {
    cmd_q: RingBufRef<Command, 4>,
    payload: [SharedSingleton<Cfg>; 2]
}

static SHARED_INTF: Interface = Interface {cmd_q: RingBufRef::INIT_0, payload: [SharedSingleton::INIT_0; 2] };

struct ProducerState{
    iter: u32,
    last_cfg_idx: u32,
}
pub fn producer_irq(idx: usize) {

    // Retrieve my persistent state from the static mut scoped to this function
    // only
    let state: &'static mut ProducerState = {
        const INIT_S: ProducerState = ProducerState { iter: 0, last_cfg_idx: 0};
        static mut S: [ProducerState;4] = [INIT_S; 4];
        unsafe { &mut S[idx] }
    };

    let alloc_res = SHARED_INTF.cmd_q.alloc();

    if let Ok(cmd) = alloc_res {

        let new_idx = state.last_cfg_idx ^ 1;
        if SHARED_INTF.payload[new_idx as usize].is_consumer_owned() {
            println!("p{} No valid command payload, skip producing command this cycle!", idx);
        }
        else {
            cmd.header.set_cmd_type(CommandType::PROCESS as u32);
            state.last_cfg_idx = new_idx;
            cmd.header.set_cfg_idx(new_idx);
            let singleton: &SharedSingleton<Cfg> = &SHARED_INTF.payload[new_idx as usize];
            let payload_ref = singleton.get_mut_ref().unwrap();
            payload_ref.id = state.iter+1;
            payload_ref.sub_cfg.arr[0] = state.iter as i32;
            state.iter = state.iter + 1;
            singleton.pass_to_consumer().unwrap();
            SHARED_INTF.cmd_q.commit().unwrap();
            println!("p{} Sent 1 command", idx);
        }
    }
    else {
        println!("p{} Command queue full, skip producing command this cycle!", idx);
    }
}

pub fn consumer_irq(idx: usize) {


    while !SHARED_INTF.cmd_q.empty() {
        let cmd = SHARED_INTF.cmd_q.peek();
        match cmd {
            Some(cmd) => {

                println!("c{} Received command type {}", idx, cmd.header.cmd_type());

                assert!(SHARED_INTF.payload[cmd.header.cfg_idx() as usize].is_consumer_owned());
                SHARED_INTF.payload[cmd.header.cfg_idx() as usize].return_to_producer().unwrap();
                SHARED_INTF.cmd_q.pop().unwrap();
                println!("c{} Consumed 1 command", idx);
            },
            None => {
                assert!(false)
            }
        }
    }
}
