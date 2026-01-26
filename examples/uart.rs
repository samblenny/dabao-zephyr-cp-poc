// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
#![no_std]
#![no_main]

extern crate dabao_baremetal_poc;

use dabao_baremetal_poc::{d11ctime, gpio, uart};
use gpio::{AF, GpioPin};

/// UART example for bao1x dabao evaluation board
///
/// Initializes UART2 and prints "hello, world!" each time the PROG button
/// (PC13) is pressed. Uses d11ctime for debouncing and uart::tick() to
/// service the DMA TX queue.

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Configure PB13 and PB14 for UART2
    gpio::set_alternate_function(GpioPin::PortB(gpio::PB13), AF::AF1);
    gpio::set_alternate_function(GpioPin::PortB(gpio::PB14), AF::AF1);

    // Configure PC13 (PROG button) as input with pull-up
    gpio::set_alternate_function(GpioPin::PortC(gpio::PC13), AF::AF0);
    gpio::disable_output(GpioPin::PortC(gpio::PC13));
    gpio::enable_pullup(GpioPin::PortC(gpio::PC13));

    // Initialize UART2
    uart::init();

    // Set d11ctime for 20 ms debounce interval
    d11ctime::set_interval(d11ctime::millis_to_cycles(20));

    loop {
        // Print hello world
        uart::write(b"hello, world!\r\n");

        // Wait for d11ctime heartbeat to change (debounce)
        let last_beat = d11ctime::read_heartbeat();
        loop {
            uart::tick();
            let beat = d11ctime::read_heartbeat();
            if beat != last_beat {
                break;
            }
        }

        // Wait until PC13 is high (button released)
        loop {
            uart::tick();
            let button_state = gpio::read_input(GpioPin::PortC(gpio::PC13));
            if button_state != 0 {
                break;
            }
        }

        // Read PC13 until button is pressed (goes low)
        loop {
            uart::tick();
            let button_state = gpio::read_input(GpioPin::PortC(gpio::PC13));
            if button_state == 0 {
                break;
            }
        }
    }
}
