#![no_std]
#![no_main]

extern crate dabao_baremetal_poc;

use dabao_baremetal_poc::{d11ctime, gpio};
use gpio::{GpioPin, AF};

/// Blinky example for bao1x dabao evaluation board
///
/// Blinks an LED wired to PB12 (+) through a 330Ω or 470Ω resistor to GND.
/// The LED toggles once per second using the D11CTIME heartbeat timer.

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Configure PB12 as GPIO output
    gpio::set_alternate_function(GpioPin::PortB(gpio::PB12), AF::AF0);
    gpio::clear(GpioPin::PortB(gpio::PB12));
    gpio::enable_output(GpioPin::PortB(gpio::PB12));

    // Set heartbeat timer for 1 second interval
    d11ctime::set_interval(d11ctime::millis_to_cycles(1000));

    let mut last_beat = d11ctime::read_heartbeat();

    loop {
        let beat = d11ctime::read_heartbeat();
        if beat != last_beat {
            // Toggle LED
            gpio::toggle(GpioPin::PortB(gpio::PB12));
            last_beat = beat;
        }
    }
}
