// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! D11CTIME heartbeat timer for bao1x dabao evaluation board
//!
//! Provides direct register access for the D11CTIME heartbeat timer.
//!
//! # Overview
//!
//! The D11CTIME timer is a simple heartbeat timer that toggles a bit at a
//! fixed interval. It's useful for deterministic timing without delay loops.
//!
//! # Registers
//!
//! - CONTROL: Write the ACLK cycle count for the desired interval.
//!   The timer will toggle the heartbeat bit when this count expires.
//!
//! - HEARTBEAT: Read-only register with heartbeat status in bit 0.
//!   Toggles each time the interval expires.
//!
//! # Example
//!
//! To set a 1-second interval at 350 MHz:
//! 1. Write 350_000_000 to CONTROL
//! 2. Poll HEARTBEAT bit 0 to detect toggles

// ============================================================================
// Constants
// ============================================================================

const D11CTIME_BASE: usize = 0xe0000000;
const CONTROL: *mut u32 = D11CTIME_BASE as *mut u32;
const HEARTBEAT: *const u32 = (D11CTIME_BASE + 4) as *const u32;

pub const ACLK_FREQ_HZ: u32 = 350_000_000;

// ============================================================================
// Register Access Functions
// ============================================================================

/// Write the interval to the control register.
///
/// Set this to the number of ACLK cycles for the desired interval.
/// At 350 MHz, 350_000_000 = 1 second.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent timer access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn set_interval(cycles: u32) {
    unsafe {
        core::ptr::write_volatile(CONTROL, cycles);
    }
}

/// Read the heartbeat bit.
///
/// Returns the current state of bit 0 of the heartbeat register.
/// Toggles each time the interval expires.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent timer access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn read_heartbeat() -> u32 {
    unsafe { core::ptr::read_volatile(HEARTBEAT) & 1 }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate ACLK cycles for a given time interval in milliseconds.
///
/// Returns the number of cycles needed for the specified millisecond
/// interval. At 350 MHz, this provides 1 ms precision.
///
/// Example: millis_to_cycles(1000) = 350,000,000 (1 second)
#[inline]
pub const fn millis_to_cycles(millis: u32) -> u32 {
    (ACLK_FREQ_HZ / 1000) * millis
}
