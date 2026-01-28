// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
#![no_std]
#![no_main]
extern crate dabao_baremetal_poc;
use dabao_baremetal_poc::{gpio, log, sleep, ticktimer, uart};
use gpio::{AF, GpioPin};

/// UART example for bao1x dabao evaluation board
///
/// Demonstrates UART2 and the ticktimer module by repeatedly printing
/// "hello, world!" with the current millisecond timestamp (from the TICKTIMER
/// peripheral). Waits for button press/release cycles on the PROG button
/// (PC13) between prints, using crate::sleep() for debouncing. Uses
/// uart::tick() to service the DMA TX queue.
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Configure PC13 (PROG button) as input with pull-up
    gpio::set_alternate_function(GpioPin::PortC(gpio::PC13), AF::AF0);
    gpio::disable_output(GpioPin::PortC(gpio::PC13));
    gpio::enable_pullup(GpioPin::PortC(gpio::PC13));

    // UART2 initialization happens at boot time in crate::init()

    loop {
        // Print message prefix
        let ms = ticktimer::millis();
        log!("hello, world! [millis() = {}]\r\n", ms);

        // Wait until PC13 is high (button released)
        while gpio::read_input(GpioPin::PortC(gpio::PC13)) == 0 {
            uart::tick();
        }
        sleep(10); // debounce

        // Wait until PC13 is low (button pressed)
        while gpio::read_input(GpioPin::PortC(gpio::PC13)) != 0 {
            uart::tick();
        }
        sleep(10); // debounce
    }
}
