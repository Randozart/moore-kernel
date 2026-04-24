#![no_std]
#![no_main]

pub mod kernel_types {

    pub struct TileInfo {
        pub tile_id: u32,
        pub lut_count: u32,
        pub dsp_count: u32,
        pub bram_mb: f64,
        pub connected: bool,
    }

    pub struct MountInfo {
        pub slot: u32,
        pub bitstream_name: Option<u32>,
        pub active: bool,
        pub lut_count: u32,
    }

    pub struct StorageInfo {
        pub filename_ptr: u32,
        pub size: u64,
        pub verified: bool,
    }

    pub struct FenceInfo {
        pub fence_id: u32,
        pub active: bool,
        pub mode: u8,
    }

    pub struct KernelInfo {
        pub version: u32,
        pub build_date: u32,
    }

    pub struct KernelState {
        pub uptime: u64,
        pub mounted_count: u32,
        pub fence_active: bool,
        pub tile_count: u32,
        pub mount_count: u32,
        pub storage_count: u32,
        pub fence_count: u32,
    }

    pub struct FenceManager {
        fences: [Option<FenceInfo>; 4],
        count: u8,
    }

    impl FenceManager {
        pub const fn new() -> Self {
            Self {
                fences: [None, None, None, None],
                count: 0,
            }
        }

        pub fn activate(&mut self, slot: u8, mode: u8) -> u8 {
            if self.count >= 4 {
                return 0x34;
            }
            self.fences[self.count as usize] = Some(FenceInfo {
                fence_id: slot as u32,
                active: true,
                mode,
            });
            self.count += 1;
            0
        }

        pub fn deactivate(&mut self, slot: u8) {
            for fence in &mut self.fences {
                if let Some(ref mut f) = fence {
                    if f.fence_id == slot as u32 {
                        f.active = false;
                        return;
                    }
                }
            }
        }

        pub fn deactivate_all(&mut self) {
            for fence in &mut self.fences {
                if let Some(ref mut f) = fence {
                    f.active = false;
                }
            }
        }

        pub fn get_status(&self, slot: u8) -> Option<(bool, u8)> {
            for fence in &self.fences {
                if let Some(ref f) = fence {
                    if f.fence_id == slot as u32 {
                        return Some((f.active, f.mode));
                    }
                }
            }
            None
        }
    }
}

use kernel_types::*;

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