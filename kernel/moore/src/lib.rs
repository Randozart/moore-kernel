// moore/lib.rs - Moore Kernel Library
//     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
//
// Moore Kernel - Bare-metal OS treating FPGA bitstreams as
// first-class physical processes on Xilinx KV260

#![no_std]
#![no_main]

pub mod boot;
pub mod kernel_types;

use kernel_types::FenceManager;

static mut TICK_COUNT: u64 = 0;
static mut FENCE_MGR: FenceManager = FenceManager::new();

pub fn init_hardware() -> u8 {
    unsafe {
        FENCE_MGR.deactivate_all();
    }
    0
}

pub fn scheduler_init() -> u8 {
    unsafe {
        TICK_COUNT = 0;
    }
    0
}

pub fn scheduler_tick() {
    unsafe {
        TICK_COUNT += 1;
    }
}

pub fn uptime() -> u64 {
    unsafe { TICK_COUNT }
}

#[no_mangle]
pub extern "C" fn moore_main() -> ! {
    let result = boot::boot();
    if !result.is_ok() {
        boot::early_puts("Boot failed: ");
        boot::early_puts(result.message);
        boot::early_puts("\n");
        loop {}
    }

    let result = init_hardware();
    if result != 0 {
        loop {}
    }

    let result = scheduler_init();
    if result != 0 {
        loop {}
    }

    loop {
        scheduler_tick();
    }
}