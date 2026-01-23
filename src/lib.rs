// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! Bare-metal initialization and hardware drivers for bao1x dabao
#![no_std]
#![no_main]

pub mod d11ctime;
pub mod gpio;

use core::arch::asm;
use core::panic::PanicInfo;

unsafe extern "C" {
    fn _data_lma(); //  Start .data in FLASH (ReRAM)
    fn _data_vma(); //  Start .data in SRAM
    fn _bss_vma(); //   Start .bss  in SRAM
    fn _data_size(); // Size of .data
    fn _bss_size(); //  Size of .bss
    fn _ram_top();
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
            "mv sp, {0}",
            "addi sp, sp, -4",  // 4 byte DMA gutter
            in(reg) _ram_top,   // get address from linker script
        );
        init();
        // Trick linker into keeping TEST_DATA
        core::ptr::write_volatile(&raw mut TEST_DATA, TEST_DATA);
        main();
    }
}

/// Copy .data and zero .bss
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
    }
}

/// Panic Handler for no_std.
#[panic_handler]
pub fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}
