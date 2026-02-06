// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! Generic wrapper for C examples using dabao_sdk drivers
//!
//! This binary calls c_main() from linked C code, allowing C examples to
//! be built with the Rust SDK initialization and driver infrastructure.
//!
//! To use this wrapper with a C example:
//! 1. Create examples/foo_c.c with an int c_main() function
//! 2. Compile C to static library with gcc and ar
//! 3. Build this wrapper with RUSTFLAGS linking the C library
//! 4. The Rust _start() initializes hardware and calls C's c_main()

#![no_std]
#![no_main]
extern crate dabao_sdk;

unsafe extern "C" {
    fn c_main() -> i32;
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        c_main();
    }
    loop {}
}
