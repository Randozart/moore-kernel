#![no_std]

use core::ptr::{read_volatile, write_volatile};

pub const PCAP_BASE: u32 = 0xFF0A_0000;

pub const PCAP_CTRL: u32 = PCAP_BASE + 0x00;
pub const PCAP_STAT: u32 = PCAP_BASE + 0x04;
pub const PCAP_ADDR: u32 = PCAP_BASE + 0x08;
pub const PCAP_SIZE: u32 = PCAP_BASE + 0x0C;
pub const PCAP_DATA: u32 = PCAP_BASE + 0x10;
pub const PCAP_LOCK: u32 = PCAP_BASE + 0x14;

pub const PCAP_CTRL_RUN: u32 = 1 << 0;
pub const PCAP_CTRL_READ: u32 = 1 << 1;
pub const PCAP_CTRL_WRITE: u32 = 1 << 2;
pub const PCAP_CTRL_RESET: u32 = 1 << 3;

pub const PCAP_STAT_FIFO_EMPTY: u32 = 1 << 2;
pub const PCAP_STAT_FIFO_FULL: u32 = 1 << 3;
pub const PCAP_STAT_BUSY: u32 = 1 << 4;

pub const PARTITION_SIZE_MAX: u32 = 8 * 1024 * 1024;

pub const ERR_OK: u8 = 0x01;
pub const ERR_NOT_FOUND: u8 = 0x02;
pub const ERR_VERIFY_FAIL: u8 = 0x03;
pub const ERR_TIMEOUT: u8 = 0x08;
pub const ERR_DFX_DECOUPLE: u8 = 0x09;

#[repr(C)]
pub struct PcapStatus {
    pub busy: bool,
    pub fifo_empty: bool,
    pub fifo_full: bool,
    pub error: u8,
}

impl PcapStatus {
    pub fn read() -> Self {
        let stat = unsafe { read_volatile(PCAP_STAT as *const u32) };
        Self {
            busy: (stat & PCAP_STAT_BUSY) != 0,
            fifo_empty: (stat & PCAP_STAT_FIFO_EMPTY) != 0,
            fifo_full: (stat & PCAP_STAT_FIFO_FULL) != 0,
            error: ((stat >> 8) & 0xFF) as u8,
        }
    }
}

#[repr(C)]
pub struct PcapTransfer {
    pub dest_addr: u32,
    pub source_ptr: *const u8,
    pub byte_count: u32,
}

impl PcapTransfer {
    pub fn new(dest: u32, src: *const u8, bytes: u32) -> Self {
        Self {
            dest_addr: dest,
            source_ptr: src,
            byte_count: bytes,
        }
    }
}

pub fn init() -> Result<(), u8> {
    let mut ctrl = PCAP_CTRL_RESET;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    ctrl = 0;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    Ok(())
}

pub fn is_busy() -> bool {
    PcapStatus::read().busy
}

pub fn wait_until_idle(timeout_cycles: u32) -> Result<(), u8> {
    let mut count = 0;
    while is_busy() {
        count += 1;
        if count >= timeout_cycles {
            return Err(ERR_TIMEOUT);
        }
    }
    Ok(())
}

pub fn write_register(addr: u32, value: u32) -> Result<(), u8> {
    if is_busy() {
        return Err(ERR_DFX_DECOUPLE);
    }
    unsafe {
        write_volatile(PCAP_ADDR as *mut u32, addr);
        write_volatile(PCAP_DATA as *mut u32, value);
    }
    let mut ctrl = PCAP_CTRL_WRITE | PCAP_CTRL_RUN;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    wait_until_idle(10000)?;
    Ok(())
}

pub fn read_register(addr: u32) -> Result<u32, u8> {
    if is_busy() {
        return Err(ERR_DFX_DECOUPLE);
    }
    unsafe {
        write_volatile(PCAP_ADDR as *mut u32, addr);
    }
    let mut ctrl = PCAP_CTRL_READ | PCAP_CTRL_RUN;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    wait_until_idle(10000)?;
    let value = unsafe { read_volatile(PCAP_DATA as *const u32) };
    Ok(value)
}

pub fn stream_bitstream(source: *const u8, byte_count: u32) -> Result<(), u8> {
    if is_busy() {
        return Err(ERR_DFX_DECOUPLE);
    }
    if byte_count > PARTITION_SIZE_MAX {
        return Err(ERR_TIMEOUT);
    }
    let mut size_reg = byte_count;
    unsafe { write_volatile(PCAP_SIZE as *mut u32, size_reg) };
    let mut ctrl = PCAP_CTRL_WRITE | PCAP_CTRL_RUN;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    wait_until_idle(1000000)?;
    Ok(())
}

pub fn get_status() -> PcapStatus {
    PcapStatus::read()
}

pub fn reset() -> Result<(), u8> {
    let mut ctrl = PCAP_CTRL_RESET;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    ctrl = 0;
    unsafe { write_volatile(PCAP_CTRL as *mut u32, ctrl) };
    Ok(())
}