// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//!
//! TIMER0 countdown timer driver
//!
//! # Hardware Details
//!
//! TIMER0 is a countdown timer in the ACLK (350 MHz) clock domain.
//! - Base address: 0xe001c000
//! - Clock: ACLK = 350 MHz
//! - Modes: One-shot (counts to 0 and stops) or periodic (auto-reload)
//! - Events: Interrupt on zero event (EV_ZERO)
//!
//! # Registers
//!
//! - LOAD (0x00): Countdown value (one-shot mode)
//! - RELOAD (0x04): Auto-reload value (periodic mode)
//! - EN (0x08): Enable (1 = running, 0 = stopped)
//! - UPDATE_VALUE (0x0c): Write to latch current countdown
//! - VALUE (0x10): Latched countdown value (read-only)
//! - EV_STATUS (0x14): Current event level (read-only)
//! - EV_PENDING (0x18): Event pending bit (write 1 to clear)
//! - EV_ENABLE (0x1c): Enable event interrupts
//!
//! # Usage
//!
//! ```ignore
//! use dabao_sdk::timer0;
//!
//! fn alarm_callback() {
//!     // Called when timer fires (in interrupt context)
//! }
//!
//! timer0::set_alarm_ms(1000, alarm_callback);
//! ```

// ====================================================================
// Callback Storage
// ====================================================================

static mut TIMER0_CALLBACK: Option<fn()> = None;

// ====================================================================
// Register Addresses
// ====================================================================

const TIMER0_LOAD: *mut u32 = 0xe001c000 as *mut u32;
const TIMER0_RELOAD: *mut u32 = 0xe001c004 as *mut u32;
const TIMER0_EN: *mut u32 = 0xe001c008 as *mut u32;
// const TIMER0_UPDATE_VALUE: *mut u32 = 0xe001c00c as *mut u32;
// const TIMER0_VALUE: *const u32 = 0xe001c010 as *const u32;
const TIMER0_EV_PENDING: *mut u32 = 0xe001c018 as *mut u32;
const TIMER0_EV_ENABLE: *mut u32 = 0xe001c01c as *mut u32;

// ====================================================================
// Public API
// ====================================================================

/// Set one-shot alarm after specified milliseconds
///
/// # Arguments
/// * `ms` - Milliseconds until alarm fires (1-4294967295)
/// * `callback` - Function to call when alarm fires (runs in interrupt context)
///
/// # Notes
/// Timer counts down from LOAD value to 0, fires event, and stops.
/// The callback will be invoked in interrupt context - keep it short
/// and avoid blocking operations.
///
/// # Example
/// ```ignore
/// fn alarm_fired() {
///     // This runs in interrupt context
/// }
/// timer0::set_alarm_ms(1000, alarm_fired);
/// ```
pub fn set_alarm_ms(ms: u32, callback: fn()) {
    // Calculate countdown value in ACLK cycles
    // cycles = (ACLK_HZ / 1000) * ms
    let cycles = (crate::ACLK_HZ / 1000).saturating_mul(ms);

    unsafe {
        // Store callback before starting timer
        TIMER0_CALLBACK = Some(callback);

        // Disable timer and zero event interrupt before reconfiguring
        core::ptr::write_volatile(TIMER0_EN, 0);
        core::ptr::write_volatile(TIMER0_EV_ENABLE, 0);

        // Clear any pending interrupt
        core::ptr::write_volatile(TIMER0_EV_PENDING, 1);

        // Ensure timer and interrupts are off before configuring
        core::sync::atomic::compiler_fence(
            core::sync::atomic::Ordering::SeqCst,
        );

        // Set countdown value (one-shot mode)
        core::ptr::write_volatile(TIMER0_LOAD, cycles);

        // Zero the reload value
        core::ptr::write_volatile(TIMER0_RELOAD, 0);

        // Enable event interrupt generation
        core::ptr::write_volatile(TIMER0_EV_ENABLE, 1);

        // Ensure timer is configured before starting
        core::sync::atomic::compiler_fence(
            core::sync::atomic::Ordering::SeqCst,
        );

        // Start timer
        core::ptr::write_volatile(TIMER0_EN, 1);
    }
}

/// Stop timer, clear pending interrupt event, disable interrupt signalling
pub fn stop_and_clear() {
    unsafe {
        core::ptr::write_volatile(TIMER0_EN, 0);
        core::ptr::write_volatile(TIMER0_EV_ENABLE, 0);
        core::ptr::write_volatile(TIMER0_EV_PENDING, 1); // write 1 to clear!
    }
}

/// Retrieve the current callback (for interrupt handler use)
pub(crate) fn get_callback() -> Option<fn()> {
    unsafe { TIMER0_CALLBACK }
}
