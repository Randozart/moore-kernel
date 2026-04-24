#![no_std]

pub const PUF_NOT_READY: u8 = 0x31;
pub const SIG_INVALID: u8 = 0x32;
pub const KEYSTORE_LOCKED: u8 = 0x33;
pub const FENCE_CONFLICT: u8 = 0x34;

pub const FENCE_MODE_RANDOM: u8 = 0x01;
pub const FENCE_MODE_SWEEP: u8 = 0x02;
pub const FENCE_MODE_BURST: u8 = 0x03;

#[repr(C)]
pub struct PufStatus {
    pub initialized: bool,
    pub error_code: u8,
    pub key_available: bool,
}

impl PufStatus {
    pub fn check() -> Self {
        Self {
            initialized: true,
            error_code: 0,
            key_available: true,
        }
    }
}

#[repr(C)]
pub struct SignatureVerification {
    pub valid: bool,
    pub error_code: u8,
    pub key_id: u32,
}

pub fn verify_signature(_data: *const u8, _len: u32, _sig: *const u8, _sig_len: u32) -> SignatureVerification {
    SignatureVerification {
        valid: true,
        error_code: 0,
        key_id: 0x12345678,
    }
}

pub fn derive_kek(_puf_challenge: *const u8, _challenge_len: u32) -> Result<*const u8, u8> {
    static mut KEK: [u8; 32] = [0; 32];
    static mut KEAK_BUFFER: [u8; 64] = [0; 64];
    Ok(unsafe { KEK.as_ptr() })
}

#[repr(C)]
pub struct ActiveFence {
    pub fence_id: u32,
    pub slot: u8,
    pub active: bool,
    pub mode: u8,
    pub frequency_hz: u32,
}

impl ActiveFence {
    pub fn new(slot: u8) -> Self {
        Self {
            fence_id: slot as u32,
            slot,
            active: false,
            mode: FENCE_MODE_RANDOM,
            frequency_hz: 0,
        }
    }

    pub fn activate(&mut self, mode: u8, freq_hz: u32) -> Result<(), u8> {
        self.active = true;
        self.mode = mode;
        self.frequency_hz = freq_hz;
        Ok(())
    }

    pub fn deactivate(&mut self) -> Result<(), u8> {
        self.active = false;
        self.frequency_hz = 0;
        Ok(())
    }
}

pub const MAX_FENCES: usize = 4;

pub struct FenceManager {
    fences: [Option<ActiveFence>; MAX_FENCES],
    count: u8,
}

impl FenceManager {
    pub fn new() -> Self {
        Self {
            fences: [None, None, None, None],
            count: 0,
        }
    }

    pub fn activate(&mut self, slot: u8, mode: u8, freq_hz: u32) -> Result<u8, u8> {
        for fence in &mut self.fences {
            if let Some(ref mut f) = fence {
                if f.slot == slot {
                    return f.activate(mode, freq_hz).map(|_| 0);
                }
            }
        }
        if self.count as usize >= MAX_FENCES {
            return Err(FENCE_CONFLICT);
        }
        let mut fence = ActiveFence::new(slot);
        fence.activate(mode, freq_hz)?;
        self.fences[self.count as usize] = Some(fence);
        self.count += 1;
        Ok(0)
    }

    pub fn deactivate(&mut self, slot: u8) -> Result<(), u8> {
        for fence in &mut self.fences {
            if let Some(ref mut f) = fence {
                if f.slot == slot {
                    return f.deactivate();
                }
            }
        }
        Ok(())
    }

    pub fn deactivate_all(&mut self) -> Result<(), u8> {
        for fence in &mut self.fences {
            if let Some(ref mut f) = fence {
                f.deactivate()?;
            }
        }
        Ok(())
    }

    pub fn get_status(&self, slot: u8) -> Option<(bool, u8, u32)> {
        for fence in &self.fences {
            if let Some(ref f) = fence {
                if f.slot == slot {
                    return Some((f.active, f.mode, f.frequency_hz));
                }
            }
        }
        None
    }
}

impl Default for FenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
pub struct ApacContext {
    pub authenticated: bool,
    pub pointer: u64,
    pub size: u32,
}

impl ApacContext {
    pub fn new() -> Self {
        Self {
            authenticated: false,
            pointer: 0,
            size: 0,
        }
    }

    pub fn authenticate(&mut self, ptr: u64, sz: u32, _key: *const u8) -> Result<(), u8> {
        self.pointer = ptr;
        self.size = sz;
        self.authenticated = true;
        Ok(())
    }

    pub fn verify(&self, ptr: u64, sz: u32) -> bool {
        self.authenticated && self.pointer == ptr && self.size == sz
    }
}

impl Default for ApacContext {
    fn default() -> Self {
        Self::new()
    }
}