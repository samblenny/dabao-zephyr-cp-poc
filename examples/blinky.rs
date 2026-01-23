#![no_std]
#![no_main]

extern crate dabao_baremetal_poc;

use dabao_baremetal_poc::{d11ctime, gpio};

/// Blinky example for bao1x dabao evaluation board
///
/// Blinks an LED wired to PB12 (+) through a 330Ω or 470Ω resistor to GND.
/// The LED toggles once per second using the D11CTIME heartbeat timer.

/// Rust entry point: _start() calls main()
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        // Configure PB12 as output
        gpio::gpio_set_output_b(gpio::PB12);

        // Set heartbeat timer for 1 second interval
        d11ctime::d11ctime_set_interval(
            d11ctime::d11ctime_seconds_to_cycles(1)
        );

        let mut last_beat = d11ctime::d11ctime_read_heartbeat() & 1;

        loop {
            let beat = d11ctime::d11ctime_read_heartbeat() & 1;
            if beat != last_beat {
                // Toggle LED
                gpio::gpio_toggle_b(gpio::PB12);
                last_beat = beat;
            }
        }
    }
}
