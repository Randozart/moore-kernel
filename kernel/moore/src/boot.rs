#![no_std]

pub const UART0_BASE: u32 = 0xFF00_0000;
pub const UART_CTRL: u32 = UART0_BASE;
pub const UART_STAT: u32 = UART0_BASE + 0x04;
pub const UART_DATA: u32 = UART0_BASE + 0x08;
pub const UART_BAUD: u32 = UART0_BASE + 0x0C;

pub const DDR_CTRL_BASE: u32 = 0xF800_0000;
pub const GPIO_BASE: u32 = 0xFF0B_0000;
pub const SDIO_BASE: u32 = 0xFF0C_0000;

pub const CRL_APB_BASE: u32 = 0xFF5E_0000;
pub const SLCR_LOCK: u32 = CRL_APB_BASE + 0x004;
pub const SLCR_UNLOCK: u32 = CRL_APB_BASE + 0x008;
pub const SLCR_LOCKSTA: u32 = CRL_APB_BASE + 0x00C;

pub const SLCR_UNLOCK_MAGIC: u32 = 0xDF0D;
pub const SLCR_LOCK_MAGIC: u32 = 0xDF0D;

pub const BOOT_SUCCESS: u32 = 0x0D00_0000;
pub const BOOT_FAIL_CLOCKS: u32 = 0x0D00_0001;
pub const BOOT_FAIL_DDR: u32 = 0x0D00_0002;
pub const BOOT_FAIL_UART: u32 = 0x0D00_0003;
pub const BOOT_FAIL_SD: u32 = 0x0D00_0004;

pub struct BootResult {
    pub code: u32,
    pub message: &'static str,
}

impl BootResult {
    pub fn success() -> Self {
        Self { code: BOOT_SUCCESS, message: "Boot complete" }
    }

    pub fn fail_clocks() -> Self {
        Self { code: BOOT_FAIL_CLOCKS, message: "Clock initialization failed" }
    }

    pub fn fail_ddr() -> Self {
        Self { code: BOOT_FAIL_DDR, message: "DDR4 initialization failed" }
    }

    pub fn fail_uart() -> Self {
        Self { code: BOOT_FAIL_UART, message: "UART initialization failed" }
    }

    pub fn fail_sd() -> Self {
        Self { code: BOOT_FAIL_SD, message: "SD card initialization failed" }
    }

    pub fn is_ok(&self) -> bool {
        self.code == BOOT_SUCCESS
    }
}

pub fn unlock_slcr() {
    unsafe {
        core::ptr::write_volatile(SLCR_UNLOCK as *mut u32, SLCR_UNLOCK_MAGIC);
    }
}

pub fn lock_slcr() {
    unsafe {
        core::ptr::write_volatile(SLCR_LOCK as *mut u32, SLCR_LOCK_MAGIC);
    }
}

pub fn init_clocks() -> BootResult {
    unlock_slcr();

    unsafe {
        let _ = core::ptr::read_volatile(CRL_APB_BASE as *const u32);
    }

    lock_slcr();
    BootResult::success()
}

pub fn init_ddr() -> BootResult {
    unsafe {
        let ddr_ctrl = core::ptr::read_volatile(DDR_CTRL_BASE as *const u32);
        if ddr_ctrl == 0 {
            return BootResult::fail_ddr();
        }
    }
    BootResult::success()
}

pub fn init_uart() -> BootResult {
    unsafe {
        core::ptr::write_volatile(UART_BAUD as *mut u32, 0x3B);
        core::ptr::write_volatile(UART_CTRL as *mut u32, 0x17);
    }
    BootResult::success()
}

pub fn init_sdcard() -> BootResult {
    unsafe {
        let sdio_ctrl = core::ptr::read_volatile(SDIO_BASE as *const u32);
        if sdio_ctrl == 0 {
            return BootResult::fail_sd();
        }
    }
    BootResult::success()
}

pub fn init_gpio() {
    unsafe {
        core::ptr::write_volatile(GPIO_BASE as *mut u32, 0);
    }
}

pub fn early_puts(s: &str) {
    let mut uart = Uart::new();
    for c in s.bytes() {
        uart.putchar(c);
    }
}

pub fn boot() -> BootResult {
    early_puts("MOORE KERNEL v0.1.0\n");
    early_puts("Initializing hardware...\n");

    if !init_clocks().is_ok() {
        early_puts("FAIL: clocks\n");
        return BootResult::fail_clocks();
    }
    early_puts("[OK] Clocks\n");

    if !init_ddr().is_ok() {
        early_puts("FAIL: DDR\n");
        return BootResult::fail_ddr();
    }
    early_puts("[OK] DDR4\n");

    if !init_uart().is_ok() {
        early_puts("FAIL: UART\n");
        return BootResult::fail_uart();
    }
    early_puts("[OK] UART\n");

    init_gpio();

    if !init_sdcard().is_ok() {
        early_puts("FAIL: SD\n");
        return BootResult::fail_sd();
    }
    early_puts("[OK] SD Card\n");

    early_puts("Hardware initialized.\n\n");
    BootResult::success()
}

pub struct Uart;

impl Uart {
    pub fn new() -> Self {
        Self
    }

    pub fn putchar(&mut self, c: u8) {
        while self.is_tx_full() {}
        unsafe { core::ptr::write_volatile(UART_DATA as *mut u32, c as u32) };
    }

    fn is_tx_full(&self) -> bool {
        let stat = unsafe { core::ptr::read_volatile(UART_STAT as *const u32) };
        (stat & 0x20) != 0
    }
}