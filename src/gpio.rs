// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! GPIO support for bao1x dabao evaluation board
//!
//! Provides direct, minimal register access for GPIO control on ports B
//! and C.
//!
//! # GPIO Access Model
//!
//! GPIO pins are controlled via memory-mapped registers. Each pin is
//! represented as a bit in a 16-bit register. Constants like `PC13` define
//! the bit mask for individual pins (e.g., `PC13 = 1 << 13`).
//!
//! # Registers
//!
//! For each port, four registers control GPIO behavior:
//!
//! - GPIOOUT: Output value register. Writing 1 sets the pin high,
//!   writing 0 sets it low. Only has effect when output enable is set.
//!
//! - GPIOOE: Output enable register. Writing 1 makes the pin an output,
//!   writing 0 makes it an input. Bit positions match GPIOOUT.
//!
//! - GPIOPU: Pull-up register. Writing 1 enables internal pull-up,
//!   writing 0 disables it. Only effective when pin is configured as input.
//!
//! - GPIOIN: Input register (read-only). Reflects the current state of
//!   pins configured as inputs.
//!
//! # API Organization
//!
//! - Functions are organized by port (`_b` for Port B, `_c` for Port C).
//! - Register access functions read/write entire registers.
//! - Bit manipulation helpers (set/clear/toggle) perform
//!   read-modify-write operations.
//! - High-level configuration functions combine multiple register
//!   operations to set pin modes (output, input, open-drain, etc.).

const GPIO_BASE: usize = 0x5012f000;

// Port B (port 1) register addresses
const GPIOOUT_B: *mut u16 = (GPIO_BASE + 0x130 + 1 * 4) as *mut u16;
const GPIOOE_B: *mut u16 = (GPIO_BASE + 0x148 + 1 * 4) as *mut u16;
const GPIOPU_B: *mut u16 = (GPIO_BASE + 0x160 + 1 * 4) as *mut u16;
const GPIOIN_B: *mut u16 = (GPIO_BASE + 0x178 + 1 * 4) as *mut u16;

// Port C (port 2) register addresses
const GPIOOUT_C: *mut u16 = (GPIO_BASE + 0x130 + 2 * 4) as *mut u16;
const GPIOOE_C: *mut u16 = (GPIO_BASE + 0x148 + 2 * 4) as *mut u16;
const GPIOPU_C: *mut u16 = (GPIO_BASE + 0x160 + 2 * 4) as *mut u16;
const GPIOIN_C: *mut u16 = (GPIO_BASE + 0x178 + 2 * 4) as *mut u16;

pub const PORT_B: u8 = 1;
pub const PORT_C: u8 = 2;

pub const PB1: u16 = 1 << 1;
pub const PB2: u16 = 1 << 2;
pub const PB3: u16 = 1 << 3;
pub const PB4: u16 = 1 << 4;
pub const PB5: u16 = 1 << 5;
pub const PB11: u16 = 1 << 11;
pub const PB12: u16 = 1 << 12;
pub const PB13: u16 = 1 << 13; // Bootloader UART RX
pub const PB14: u16 = 1 << 14; // Bootloader UART TX

pub const PC0: u16 = 1 << 0;
pub const PC1: u16 = 1 << 1;
pub const PC2: u16 = 1 << 2;
pub const PC3: u16 = 1 << 3;
pub const PC7: u16 = 1 << 7;
pub const PC8: u16 = 1 << 8;
pub const PC9: u16 = 1 << 9;
pub const PC10: u16 = 1 << 10;
pub const PC11: u16 = 1 << 11;
pub const PC12: u16 = 1 << 12;
pub const PC13: u16 = 1 << 13;

// ============================================================================
// Register Access Functions - Port B
// ============================================================================

#[inline]
pub unsafe fn gpio_read_output_b() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOOUT_B) }
}

#[inline]
pub unsafe fn gpio_write_output_b(value: u16) {
    unsafe { core::ptr::write_volatile(GPIOOUT_B, value); }
}

#[inline]
pub unsafe fn gpio_read_oe_b() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOOE_B) }
}

#[inline]
pub unsafe fn gpio_write_oe_b(value: u16) {
    unsafe { core::ptr::write_volatile(GPIOOE_B, value); }
}

#[inline]
pub unsafe fn gpio_read_pu_b() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOPU_B) }
}

#[inline]
pub unsafe fn gpio_write_pu_b(value: u16) {
    unsafe { core::ptr::write_volatile(GPIOPU_B, value); }
}

#[inline]
pub unsafe fn gpio_read_input_b() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOIN_B) }
}

// ============================================================================
// Register Access Functions - Port C
// ============================================================================

#[inline]
pub unsafe fn gpio_read_output_c() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOOUT_C) }
}

#[inline]
pub unsafe fn gpio_write_output_c(value: u16) {
    unsafe { core::ptr::write_volatile(GPIOOUT_C, value); }
}

#[inline]
pub unsafe fn gpio_read_oe_c() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOOE_C) }
}

#[inline]
pub unsafe fn gpio_write_oe_c(value: u16) {
    unsafe { core::ptr::write_volatile(GPIOOE_C, value); }
}

#[inline]
pub unsafe fn gpio_read_pu_c() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOPU_C) }
}

#[inline]
pub unsafe fn gpio_write_pu_c(value: u16) {
    unsafe { core::ptr::write_volatile(GPIOPU_C, value); }
}

#[inline]
pub unsafe fn gpio_read_input_c() -> u16 {
    unsafe { core::ptr::read_volatile(GPIOIN_C) }
}

// ============================================================================
// Bit Manipulation Helpers - Port B
// ============================================================================

#[inline]
pub unsafe fn gpio_set_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOUT_B);
        core::ptr::write_volatile(GPIOOUT_B, current | mask);
    }
}

#[inline]
pub unsafe fn gpio_clear_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOUT_B);
        core::ptr::write_volatile(GPIOOUT_B, current & !mask);
    }
}

#[inline]
pub unsafe fn gpio_toggle_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOUT_B);
        core::ptr::write_volatile(GPIOOUT_B, current ^ mask);
    }
}

#[inline]
pub unsafe fn gpio_set_oe_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOE_B);
        core::ptr::write_volatile(GPIOOE_B, current | mask);
    }
}

#[inline]
pub unsafe fn gpio_clear_oe_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOE_B);
        core::ptr::write_volatile(GPIOOE_B, current & !mask);
    }
}

#[inline]
pub unsafe fn gpio_set_pu_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOPU_B);
        core::ptr::write_volatile(GPIOPU_B, current | mask);
    }
}

#[inline]
pub unsafe fn gpio_clear_pu_bits_b(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOPU_B);
        core::ptr::write_volatile(GPIOPU_B, current & !mask);
    }
}

// ============================================================================
// Bit Manipulation Helpers - Port C
// ============================================================================

#[inline]
pub unsafe fn gpio_set_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOUT_C);
        core::ptr::write_volatile(GPIOOUT_C, current | mask);
    }
}

#[inline]
pub unsafe fn gpio_clear_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOUT_C);
        core::ptr::write_volatile(GPIOOUT_C, current & !mask);
    }
}

#[inline]
pub unsafe fn gpio_toggle_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOUT_C);
        core::ptr::write_volatile(GPIOOUT_C, current ^ mask);
    }
}

#[inline]
pub unsafe fn gpio_set_oe_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOE_C);
        core::ptr::write_volatile(GPIOOE_C, current | mask);
    }
}

#[inline]
pub unsafe fn gpio_clear_oe_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOOE_C);
        core::ptr::write_volatile(GPIOOE_C, current & !mask);
    }
}

#[inline]
pub unsafe fn gpio_set_pu_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOPU_C);
        core::ptr::write_volatile(GPIOPU_C, current | mask);
    }
}

#[inline]
pub unsafe fn gpio_clear_pu_bits_c(mask: u16) {
    unsafe {
        let current = core::ptr::read_volatile(GPIOPU_C);
        core::ptr::write_volatile(GPIOPU_C, current & !mask);
    }
}

// ============================================================================
// High-Level GPIO Configuration Functions - Port B
// ============================================================================

#[inline]
pub unsafe fn gpio_set_output_b(pin_mask: u16) {
    unsafe { gpio_set_oe_bits_b(pin_mask); }
}

#[inline]
pub unsafe fn gpio_set_input_floating_b(pin_mask: u16) {
    unsafe {
        gpio_clear_oe_bits_b(pin_mask);
        gpio_clear_pu_bits_b(pin_mask);
    }
}

#[inline]
pub unsafe fn gpio_set_input_pullup_b(pin_mask: u16) {
    unsafe {
        gpio_clear_oe_bits_b(pin_mask);
        gpio_set_pu_bits_b(pin_mask);
    }
}

#[inline]
pub unsafe fn gpio_set_open_drain_b(pin_mask: u16) {
    unsafe { gpio_set_oe_bits_b(pin_mask); }
}

#[inline]
pub unsafe fn gpio_set_high_b(pin_mask: u16) {
    unsafe { gpio_set_bits_b(pin_mask); }
}

#[inline]
pub unsafe fn gpio_set_low_b(pin_mask: u16) {
    unsafe { gpio_clear_bits_b(pin_mask); }
}

#[inline]
pub unsafe fn gpio_toggle_b(pin_mask: u16) {
    unsafe { gpio_toggle_bits_b(pin_mask); }
}

// ============================================================================
// High-Level GPIO Configuration Functions - Port C
// ============================================================================

#[inline]
pub unsafe fn gpio_set_output_c(pin_mask: u16) {
    unsafe { gpio_set_oe_bits_c(pin_mask); }
}

#[inline]
pub unsafe fn gpio_set_input_floating_c(pin_mask: u16) {
    unsafe {
        gpio_clear_oe_bits_c(pin_mask);
        gpio_clear_pu_bits_c(pin_mask);
    }
}

#[inline]
pub unsafe fn gpio_set_input_pullup_c(pin_mask: u16) {
    unsafe {
        gpio_clear_oe_bits_c(pin_mask);
        gpio_set_pu_bits_c(pin_mask);
    }
}

#[inline]
pub unsafe fn gpio_set_open_drain_c(pin_mask: u16) {
    unsafe { gpio_set_oe_bits_c(pin_mask); }
}

#[inline]
pub unsafe fn gpio_set_high_c(pin_mask: u16) {
    unsafe { gpio_set_bits_c(pin_mask); }
}

#[inline]
pub unsafe fn gpio_set_low_c(pin_mask: u16) {
    unsafe { gpio_clear_bits_c(pin_mask); }
}

#[inline]
pub unsafe fn gpio_toggle_c(pin_mask: u16) {
    unsafe { gpio_toggle_bits_c(pin_mask); }
}
