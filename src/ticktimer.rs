// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! TICKTIMER driver for bao1x dabao evaluation board
//!
//! Provides a system millisecond timer via the TICKTIMER peripheral.
//!
//! # Overview
//!
//! TICKTIMER is a 64-bit counter that increments at a configurable rate.
//! This driver configures it to increment once per millisecond, providing
//! a system-wide millisecond clock accessible via `millis()`.
//!
//! # Usage
//!
//! The timer is initialized automatically at boot time. Application code
//! can read the current time in milliseconds:
//!
//! ```ignore
//! use dabao_sdk::ticktimer;
//!
//! fn main() {
//!     let start = ticktimer::millis();
//!     // ... do some work ...
//!     let elapsed = ticktimer::millis() - start;
//! }
//! ```
//!
//! # Clock Configuration
//!
//! TICKTIMER increments based on CLOCKS_PER_TICK:
//! - At 350 MHz ACLK, CLOCKS_PER_TICK = 350,000 gives 1ms ticks
//! - Formula: CLOCKS_PER_TICK = ACLK_HZ / 1000
//!
//! The current default (800,000) was designed for 800 MHz systems.
//! We override it to 350,000 for the Bao1x at 350 MHz.
//!
//! # Registers
//!
//! TICKTIMER provides:
//! - TIME1/TIME0: 64-bit elapsed time in millisecond ticks
//! - CONTROL: Reset control
//! - CLOCKS_PER_TICK: Divisor for tick rate
//! - Event control registers (not used in this driver)

use core::ptr;

// ============================================================================
// Constants
// ============================================================================

// TICKTIMER register addresses
const TICKTIMER_TIME0: *const u32 = 0xe001b008 as *const u32;
const TICKTIMER_TIME1: *const u32 = 0xe001b004 as *const u32;
const TICKTIMER_CLOCKS_PER_TICK: *mut u32 = 0xe001b020 as *mut u32;

// Calculate clocks per millisecond from system clock frequency
const CLOCKS_PER_MS: u32 = crate::ACLK_HZ / 1000;

// ============================================================================
// Public API
// ============================================================================

/// Initialize the timer for 1 millisecond tick rate.
///
/// Sets CLOCKS_PER_TICK to (ACLK_HZ / 1000) so that the timer
/// increments once per millisecond. Must be called once at boot time
/// before any code calls `millis()`.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent timer access from multiple threads would cause data races, but
/// that is not possible in this environment.
pub fn init() {
    unsafe {
        // Configure timer for 1ms ticks: 350,000 clocks per tick
        // At 350 MHz: 350,000 / 350,000,000 = 0.001 seconds = 1 millisecond
        ptr::write_volatile(TICKTIMER_CLOCKS_PER_TICK, CLOCKS_PER_MS);
        // Ensure timer configuration is complete before any millis() calls
        core::sync::atomic::compiler_fence(
            core::sync::atomic::Ordering::SeqCst,
        );
    }
}

/// Read the current elapsed time in milliseconds since boot.
///
/// Returns a u64 millisecond counter that increments at 1ms intervals.
/// The counter will not overflow for approximately 584 million years,
/// so wraparound is not a practical concern for embedded applications.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent timer access from multiple threads would cause data races, but
/// that is not possible in this environment.
pub fn millis() -> u64 {
    unsafe {
        // Read TIME0 (bits 0-31) first, then TIME1 (bits 32-63)
        // This is the safe pattern for reading split 64-bit values.
        // If TIME0 wraps while we're reading, we catch it on the next call.
        let lo = ptr::read_volatile(TICKTIMER_TIME0) as u64;
        let hi = ptr::read_volatile(TICKTIMER_TIME1) as u64;
        (hi << 32) | lo
    }
}
