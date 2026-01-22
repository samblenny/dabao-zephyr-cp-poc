// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! This module provides bare-metal initialization and hardware drivers.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

unsafe extern "C" {
    fn _idata_start(); // Start .data in FLASH (ReRAM)
    fn _data_start(); // Start .data in SRAM
    fn _data_end(); // End   .data in SRAM
    fn _bss_start(); // Start .bss  in SRAM
    fn _bss_end(); // End   .bss  in SRAM
    fn _stack_end();
    fn main() -> !;
}

/// Boot entry point (bootloader jumps here)
#[unsafe(link_section = ".text.init")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Set stack pointer to end of the stack section (from link.x).
    // Note: stack grows downward from the end of RAM.
    unsafe {
        asm!(
            "mv sp, {0}",
            "addi sp, sp, -4",   // 4 byte DMA gutter
            in(reg) _stack_end,  // get address from linker script
        );
    }

    init();
    unsafe {
        main();
    }
}

/// Copy .data and zero .bss
fn init() {
    unsafe {
        // Copy .data section from FLASH to RAM
        let data_start = _data_start as *mut u8;
        let data_end = _data_end as *mut u8;
        let flash_start = _idata_start as *const u8;
        let size = data_end as usize - data_start as usize;
        core::ptr::copy_nonoverlapping(flash_start, data_start, size);

        // Zero the .bss section
        let bss_start = _bss_start as *mut u8;
        let bss_end = _bss_end as *mut u8;
        let size = bss_end as usize - bss_start as usize;
        core::ptr::write_bytes(bss_start, 0, size);
    }
}

/// Panic Handler for no_std.
#[panic_handler]
pub fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}
