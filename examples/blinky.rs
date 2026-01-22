#![no_std]
#![no_main]
/// Goal: Blink an LED on one of the GPIO pins
extern crate dabao_baremetal_poc;

/// Rust entry point: _start() calls main()
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {
        // TODO: blinky code goes here
    }
}
