// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! Bare-metal initialization and hardware drivers for bao1x dabao
#![no_std]
#![no_main]

// System clock frequency (ACLK domain)
pub const ACLK_HZ: u32 = 350_000_000;

pub mod d11ctime;
pub mod gpio;
pub mod interrupt;
pub mod log;
pub mod ticktimer;
pub mod timer0;
pub mod uart;

use core::arch::asm;
use core::panic::PanicInfo;
use gpio::{AF, GpioPin};

unsafe extern "C" {
    fn _data_lma(); //  Start .data in FLASH (ReRAM)
    fn _data_vma(); //  Start .data in SRAM
    fn _bss_vma(); //   Start .bss  in SRAM
    fn _data_size(); // Size of .data
    fn _bss_size(); //  Size of .bss
    fn __global_pointer();
    fn _stack_base();
    fn main() -> !;
}

// This exists to help verify .data is linked properly
#[allow(dead_code)]
static mut TEST_DATA: u32 = 0x41544144; // look for "DATA" in hexdump

/// Boot entry point (bootloader jumps here)
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Set stack pointer to end of the stack section (from link.x).
    // Note: stack grows downward from the end of RAM.
    unsafe {
        asm!(
            "la gp, {gp_sym}", // load global pointer from linker script
            "la sp, {sp_sym}",
            "addi sp, sp, -4",  // 4 byte DMA gutter
            gp_sym = sym __global_pointer,
            sp_sym = sym _stack_base,
            options(nostack, nomem),
        );
        init();
        // Trick linker into keeping TEST_DATA
        core::ptr::write_volatile(&raw mut TEST_DATA, TEST_DATA);
        main();
    }
}

/// Sleep for specified milliseconds, servicing UART DMA.
///
/// Blocks until the specified time has elapsed, calling uart::tick()
/// periodically to service the TX DMA queue.
#[unsafe(no_mangle)]
pub extern "C" fn sleep(ms: u32) {
    let end_time = ticktimer::millis() + ms as u64;
    while ticktimer::millis() < end_time {
        uart::tick();
    }
}

/// Initialize system state and peripherals at boot.
///
/// Copies .data section from FLASH to RAM, zeros .bss section, and
/// initializes peripherals (timer and other drivers).
fn init() {
    unsafe {
        // Copy .data section from FLASH to RAM
        let src = _data_lma as *const u8;
        let dest = _data_vma as *mut u8;
        let size = _data_size as *const u8 as usize;
        core::ptr::copy_nonoverlapping(src, dest, size);

        // Zero the .bss section
        let start = _bss_vma as *mut u8;
        let size = _bss_size as *const u8 as usize;
        core::ptr::write_bytes(start, 0, size);

        // Configure PB13 and PB14 for UART2
        gpio::set_alternate_function(GpioPin::PortB(gpio::PB13), AF::AF1);
        gpio::set_alternate_function(GpioPin::PortB(gpio::PB14), AF::AF1);

        // Initialize UART first so it's available for debug output
        uart::init();

        // Initialize system timer
        ticktimer::init();

        // Initialize interrupt handler
        interrupt::irq_setup();
    }
}

/// Panic Handler for no_std.
#[panic_handler]
pub fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}
