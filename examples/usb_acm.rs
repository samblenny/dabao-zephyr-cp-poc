// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
#![no_std]
#![no_main]
extern crate baochip_sdk;
use baochip_sdk::{log, sleep, usb};

/// USB ACM serial example for bao1x dabao evaluation board
///
/// TODO: finish this comment
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    log!("starting IRQARRAY1_EV_PENDING test\r\n");
    sleep(20);
    usb::pending_write_test();
    sleep(20);
    log!("made it past pending_write_test().\r\n");
    sleep(20);
    log!("calling usb::detect().\r\n");
    sleep(20);
    usb::detect();
    log!("made it past detect().\r\n");

    loop {
        sleep(10);
    }
}
