// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! UART2 driver for bao1x dabao evaluation board
//!
//! Provides non-blocking, DMA-based serial I/O for debug logging and REPL
//! shells. TX uses buffered DMA transfers, RX uses direct polling via the
//! VALID register.
//!
//! # Usage
//!
//! Configure GPIO pins as alternate function AF1, initialize UART, then
//! read and echo characters:
//!
//! ```ignore
//! use baochip_sdk::{gpio, uart};
//! use gpio::{AF, GpioPin};
//!
//! fn main() {
//!     // Configure UART2 pins
//!     gpio::set_alternate_function(GpioPin::PortB(gpio::PB13), AF::AF1);
//!     gpio::set_alternate_function(GpioPin::PortB(gpio::PB14), AF::AF1);
//!
//!     // Initialize UART
//!     uart::init();
//!
//!     // Main loop: read a character and echo it back
//!     uart::write(b"UART echo is ready\r\n");
//!
//!     loop {
//!         if let Some(byte) = uart::getc() {
//!             uart::write(&[byte]);
//!         }
//!         uart::tick();  // Service TX DMA queue
//!     }
//! }
//! ```
//!
//! # Hardware Details
//!
//! UART2 is hardwired to GPIO pins:
//! - PB13: RX (input)
//! - PB14: TX (output)
//! These pin assignments are fixed by the hardware and cannot be changed.
//! The GPIO module must configure these pins as alternate function AF1
//! before UART2 can communicate.
//!
//! # Clock Configuration
//!
//! UART2 is clocked by PERCLK (100 MHz). Baud rate divisor is computed:
//! `divisor = PERCLK / baud_rate = 100_000_000 / 1_000_000 = 100`
//!
//! The UART_SETUP register format (bits [31:16] contain divisor):
//! `UART_SETUP = 0x0316 | (divisor << 16)`
//! - Bits [31:16]: Baud divisor (100 for 1 Mbps)
//! - Bit 9: Enable RX
//! - Bit 8: Enable TX
//! - Bit 4: RX polling mode
//! - Bit 3: Stop bits (0 = 1 stop bit)
//! - Bits [2:1]: Data bits (11 = 8 bits)
//! - Bit 0: Parity enable (0 = no parity)
//! For 8N1: 0x0316
//!
//! # TX DMA and Memory Layout
//!
//! TX data is buffered in IFRAM0 (0x50000000 - 0x5001FFFF). The 2KB TX
//! buffer is divided into 16x128-byte blocks. Each write() call fills one
//! or more blocks sequentially. Blocks that fill become ready for DMA.
//! tick() starts DMA transfers for ready blocks.
//!
//! # RX Design
//!
//! RX does not use DMA or internal buffering. getc() directly polls the
//! VALID register and reads bytes one at a time.
//!
//! # API Design
//!
//! - init(): Set up UART2 and initial state
//! - write(): Buffer TX data (non-blocking, silent drop if full)
//! - getc(): Read one byte from RX if available
//! - tick(): Start DMA for ready TX blocks

use crate::interrupt;
use core::ptr;
use core::slice;

// ============================================================================
// Constants
// ============================================================================

// UART2 register addresses
const REG_TX_SADDR: *mut u32 = 0x50103010 as *mut u32;
const REG_TX_SIZE: *mut u32 = 0x50103014 as *mut u32;
const REG_TX_CFG: *mut u32 = 0x50103018 as *mut u32;
const REG_UART_SETUP: *mut u32 = 0x50103024 as *mut u32;
const REG_VALID: *mut u32 = 0x50103030 as *mut u32;
const REG_DATA: *mut u32 = 0x50103034 as *mut u32;

// uDMA Control register addresses
const UDMA_REG_CG: *mut u32 = 0x50100000 as *mut u32;

// Peripheral IDs (bit masks)
const UART2_CLK_BIT: u32 = 1 << 2;

// TX/RX configuration bits
const CFG_EN: u32 = 1 << 4;

// UART_SETUP register bits
const UART_EN_TX: u32 = 1 << 8;
const UART_EN_RX: u32 = 1 << 9;

// VALID register bits
const VALID_DATA_AVAILABLE: u32 = 1 << 0;

// TX buffer configuration
const IFRAM_TX_ADDR: usize = 0x50000000;
const TX_BLOCK_SIZE: usize = 128;
const TX_BLOCK_COUNT: usize = 16;

// UART configuration: 8N1, 1 Mbps
const PERCLK_HZ: u32 = 100_000_000;
const UART_BAUD: u32 = 1_000_000;
const UART_DIVISOR: u32 = PERCLK_HZ / UART_BAUD;
const UART_SETUP_VALUE: u32 = 0x0316 | (UART_DIVISOR << 16);

// ============================================================================
// Internal State
// ============================================================================

// TX buffer implemented as a circular FIFO of 128-byte blocks.
// TX_NEXT_BLOCK points to the block being filled by write().
// TX_QUEUE_HEAD points to the oldest block ready for DMA.
// TX_BLOCK_LEN[i] stores the actual byte count for block i (0 = empty/done).
// When TX_NEXT_BLOCK == TX_QUEUE_HEAD and both have pending data, the
// buffer is full.
static mut TX_NEXT_BLOCK: usize = 0; // Block index for next write()
static mut TX_BLOCK_LEN: [u8; TX_BLOCK_COUNT] = [0; 16];
static mut TX_QUEUE_HEAD: usize = 0; // Block index for next DMA
static mut TX_IN_FLIGHT: bool = false; // DMA transfer active

// ============================================================================
// C API Convenience Functions
// ============================================================================

/// Expose uart::write as an extern C function for C linkage.
///
/// Takes a null-terminated string and writes it to the UART.
#[unsafe(no_mangle)]
pub extern "C" fn uart_write(data: *const u8) {
    // Safety: data pointer must not be null
    if data.is_null() {
        return;
    }

    // Find the length of the null-terminated string (max 2048 bytes)
    let mut length = 0;
    for i in 0..2048 {
        unsafe {
            if *data.add(i) == b'\0' {
                length = i;
                break;
            }
        }
    }

    // Slice the raw pointer into a Rust slice
    let slice = unsafe { slice::from_raw_parts(data, length) };

    write(slice);
}

// ============================================================================
// Public API
// ============================================================================

/// Initialize UART2 for 8N1 at 1 Mbps.
///
/// Enables the UART2 clock and configures the UART_SETUP register.
/// This assumes the bootloader has taken care of resetting the UART.
///
/// GPIO pins PB13 and PB14 must be configured separately via the GPIO
/// module as alternate function AF1 before UART2 can communicate.
///
/// Must be called before any other UART functions.
pub fn init() {
    unsafe {
        // Enable UART2 clock via uDMA control
        let cg = ptr::read_volatile(UDMA_REG_CG);
        ptr::write_volatile(UDMA_REG_CG, cg | UART2_CLK_BIT);
        // Ensure clock enable completes before configuring UART
        core::sync::atomic::compiler_fence(
            core::sync::atomic::Ordering::SeqCst,
        );

        // Configure UART_SETUP for 8N1, 1 Mbps
        // The bootloader has already reset the UART, so we just configure it.
        ptr::write_volatile(
            REG_UART_SETUP,
            UART_SETUP_VALUE | UART_EN_TX | UART_EN_RX,
        );

        // Initialize TX buffer state
        TX_NEXT_BLOCK = 0;
        TX_QUEUE_HEAD = 0;
        TX_IN_FLIGHT = false;
        for i in 0..TX_BLOCK_COUNT {
            TX_BLOCK_LEN[i] = 0;
        }
    }
}

/// Queue data for transmission via DMA.
///
/// Each call to write() uses a fresh block (or multiple blocks if data is
/// large). Does not continue filling a block from a previous write() call.
/// When a block fills (128 bytes) or when this call ends, it becomes
/// eligible for DMA. Returns the number of bytes actually buffered. If no
/// fresh blocks are available, remaining data is silently dropped.
///
/// Non-blocking - returns immediately. Starts DMA if TX is idle.
pub fn write(data: &[u8]) -> usize {
    let was_enabled = interrupt::disable_irqs();
    let mut written = 0;
    unsafe {
        let mut block = TX_NEXT_BLOCK;
        let mut offset: usize = 0;

        // Check if starting block has pending data
        if TX_BLOCK_LEN[block] > 0 {
            // Block is full and waiting to be sent, can't write
            return 0;
        }

        for &byte in data {
            // Check if current block is full
            if offset >= TX_BLOCK_SIZE {
                // Mark block as ready and move to next
                TX_BLOCK_LEN[block] = TX_BLOCK_SIZE as u8;
                block = (block + 1) % TX_BLOCK_COUNT;
                offset = 0;

                // Check if next block is available (not pending or in-flight)
                if TX_BLOCK_LEN[block] > 0 {
                    // Block has pending data, buffer is full
                    break;
                }
            }

            // Write byte to current block
            let addr = IFRAM_TX_ADDR + block * TX_BLOCK_SIZE + offset;
            ptr::write_volatile(addr as *mut u8, byte);
            offset += 1;
            written += 1;
        }

        // Record how many bytes are in the current block
        if offset > 0 {
            TX_BLOCK_LEN[block] = offset as u8;
            // Next write() will use a fresh block
            TX_NEXT_BLOCK = (block + 1) % TX_BLOCK_COUNT;
            // Ensure block state is visible to tick() before returning
            core::sync::atomic::compiler_fence(
                core::sync::atomic::Ordering::Release,
            );
        }

        // If TX is idle, start DMA for any ready blocks
        if !TX_IN_FLIGHT {
            tick();
        }
    }
    if was_enabled {
        interrupt::enable_irqs();
    }
    written
}

/// Read one byte from RX if available.
///
/// Directly polls the VALID register. Returns Some(byte) if data is
/// available, None otherwise. Non-blocking.
#[inline]
pub fn getc() -> Option<u8> {
    unsafe {
        if (ptr::read_volatile(REG_VALID) & VALID_DATA_AVAILABLE) != 0 {
            Some(ptr::read_volatile(REG_DATA) as u8)
        } else {
            None
        }
    }
}

/// Service TX DMA queue.
///
/// Checks if the current DMA transfer is complete. If so, advances the
/// queue head and starts DMA for the next ready block if available.
///
/// Call periodically from the main event loop. Also called automatically
/// by write() when needed.
pub extern "C" fn tick() {
    let was_enabled = interrupt::disable_irqs();
    unsafe {
        // Ensure we see the latest DMA state
        core::sync::atomic::compiler_fence(
            core::sync::atomic::Ordering::Acquire,
        );
        // Check if current transfer is complete
        let tx_saddr = ptr::read_volatile(REG_TX_SADDR);
        if tx_saddr == 0 && TX_IN_FLIGHT {
            // Transfer complete, mark this block as done
            TX_BLOCK_LEN[TX_QUEUE_HEAD] = 0;
            TX_QUEUE_HEAD = (TX_QUEUE_HEAD + 1) % TX_BLOCK_COUNT;
            TX_IN_FLIGHT = false;
        }

        // If idle, start DMA for next ready block
        if !TX_IN_FLIGHT && TX_QUEUE_HEAD != TX_NEXT_BLOCK {
            let len = TX_BLOCK_LEN[TX_QUEUE_HEAD];
            if len > 0 {
                let addr =
                    (IFRAM_TX_ADDR + TX_QUEUE_HEAD * TX_BLOCK_SIZE) as u32;
                ptr::write_volatile(REG_TX_SADDR, addr);
                ptr::write_volatile(REG_TX_SIZE, len as u32);
                ptr::write_volatile(REG_TX_CFG, CFG_EN);
                TX_IN_FLIGHT = true;
            }
        }
    }
    if was_enabled {
        interrupt::enable_irqs();
    }
}
