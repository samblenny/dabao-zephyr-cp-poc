// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//!
//! TIMER0 interrupt callback example for bao1x dabao evaluation board
//!
//! Demonstrates using TIMER0 with interrupt-driven callbacks. Press the PROG
//! button (PC13) to trigger a 2-second timer. When the timer fires, the
//! callback runs in interrupt context and prints "boop" with the current time.
//!
//! # Hardware Setup
//!
//! - PC13: PROG button (hardwired, active low with pull-up)
//! - TIMER0: Counts down at ACLK (350 MHz), fires interrupt at zero
//!
//! # Output Example
//!
//! ```text
//! beep 5799...boop 7798
//! beep 12224...boop 14224
//! ```
//!
//! First "beep" prints at startup. "boop" fires ~2000ms later via interrupt
//! callback. Subsequent beep/boop pairs are triggered by button presses.
//!
//! # Key Points
//!
//! - Callback runs in interrupt context - keep it short!
//! - timer0::set_alarm_ms() handles all initialization
//! - uart::tick() services UART DMA while waiting for button
//! - Timestamps show reliable interrupt timing

#![no_std]
#![no_main]
extern crate dabao_sdk;
use dabao_sdk::{gpio, log, sleep, ticktimer, timer0, uart};
use gpio::{AF, GpioPin};
use ticktimer::millis;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Configure PC13 (PROG button) as input with pull-up
    gpio::set_alternate_function(GpioPin::PortC(gpio::PC13), AF::AF0);
    gpio::disable_output(GpioPin::PortC(gpio::PC13));
    gpio::enable_pullup(GpioPin::PortC(gpio::PC13));

    loop {
        // Print timestamp and start timer for interrupt callback
        log!("beep {}...", millis());

        // Arm 2-second alarm - callback will fire in interrupt context
        timer0::set_alarm_ms(2000, alarm_callback);

        // Wait for button release (active low, so 0=pressed, 1=released)
        while gpio::read_input(GpioPin::PortC(gpio::PC13)) == 0 {
            uart::tick();
        }
        sleep(10);

        // Wait for button press
        while gpio::read_input(GpioPin::PortC(gpio::PC13)) != 0 {
            uart::tick();
        }
        sleep(10);
    }
}

/// Callback invoked when TIMER0 alarm fires (runs in interrupt context!)
///
/// This runs in interrupt context, so it must be fast and avoid blocking.
/// The callback is stored in timer0 and invoked by the trap handler when
/// the timer reaches zero.
fn alarm_callback() {
    log!("boop {}\r\n", millis());
}
