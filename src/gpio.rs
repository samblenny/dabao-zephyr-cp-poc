// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! GPIO support for bao1x dabao evaluation board
//!
//! Provides direct, minimal register access for GPIO control on ports B
//! and C. Pin operations are type-safe: each pin constant (PB13, PC3, etc.)
//! is bound to its port, preventing accidental port mismatches at compile
//! time.
//!
//! # Hardware Register Names
//!
//! The hardware registers for GPIO functionality are called IOX in the
//! Baochip documentation, but we use the more intuitive name "gpio"
//! throughout this module.
//!
//! # Unimplemented Features
//!
//! The GPIO hardware supports several additional features that are not yet
//! exposed by this driver:
//!
//! - Drive strength control (GPIOCFG_DRVSEL) - configure output drive
//!   current (2mA, 4mA, 8mA, 12mA)
//! - Slew rate control (GPIOCFG_RATCLR) - slow down output transitions
//! - Schmitt trigger (GPIOCFG_SCHM) - add hysteresis to inputs
//! - Interrupt support (INTCR, INTFR) - generate CPU interrupts on pin
//!   state changes
//!
//! These features can be added as needed. The current implementation focuses
//! on basic GPIO output, input, and alternate function operations.
//!
//! # Registers
//!
//! For each port, four registers control GPIO behavior:
//!
//! - GPIOOUT: Output value register. Writing 1 sets the pin high,
//!   writing 0 sets it low. Only has effect when output enable is set.
//!
//! - GPIOOE: Output enable register. Writing 1 makes the pin an output,
//!   writing 0 makes it an input.
//!
//! - GPIOPU: Pull-up register. Writing 1 enables internal pull-up,
//!   writing 0 disables it. Only effective when pin is configured as input.
//!
//! - GPIOIN: Input register (read-only). Reflects the current state of
//!   pins configured as inputs.
//!
//! # Usage Examples
//!
//! Configure the PROG button input (PC13):
//! ```ignore
//! use gpio::{GpioPin, AF};
//!
//! gpio::set_alternate_function(GpioPin::PortC(gpio::PC13), AF::AF0);
//! gpio::disable_output(GpioPin::PortC(gpio::PC13));
//! gpio::enable_pullup(GpioPin::PortC(gpio::PC13));
//! ```
//!
//! Configure an LED output pin (PB12):
//! The dabao board does not have a built-in LED. To test output, wire an LED
//! to PB12 with a current-limiting resistor (330Ω or 470Ω) to GND:
//! ```ignore
//! use gpio::{GpioPin, AF};
//!
//! gpio::set_alternate_function(GpioPin::PortB(gpio::PB12), AF::AF0);
//! gpio::enable_output(GpioPin::PortB(gpio::PB12));
//! gpio::set(GpioPin::PortB(gpio::PB12));
//! ```
//!
//! # API Design
//!
//! The public API consists of:
//! - `set()`: Set pin output high
//! - `clear()`: Set pin output low
//! - `toggle()`: Toggle pin output
//! - `enable_output()`: Configure pin as output
//! - `disable_output()`: Configure pin as input
//! - `enable_pullup()`: Enable internal pull-up
//! - `disable_pullup()`: Disable internal pull-up
//! - `read_input()`: Read current input state of a pin
//! - `set_alternate_function()`: Configure pin for peripheral functions

pub struct PortBPin(u16);
pub struct PortCPin(u16);

pub enum GpioPin {
    PortB(PortBPin),
    PortC(PortCPin),
}

pub enum AF {
    AF0 = 0, // GPIO (default)
    AF1 = 1, // UART2, I2C0, I2C1, CAM, SPIM2
    AF2 = 2, // SDIO, SPIM1, I2SS, I2SM, SPIS
    AF3 = 3, // Timer PWM outputs
}

pub const PB1: PortBPin = PortBPin(1 << 1);
pub const PB2: PortBPin = PortBPin(1 << 2);
pub const PB3: PortBPin = PortBPin(1 << 3);
pub const PB4: PortBPin = PortBPin(1 << 4);
pub const PB5: PortBPin = PortBPin(1 << 5);
pub const PB11: PortBPin = PortBPin(1 << 11);
pub const PB12: PortBPin = PortBPin(1 << 12);
pub const PB13: PortBPin = PortBPin(1 << 13);
pub const PB14: PortBPin = PortBPin(1 << 14);

pub const PC0: PortCPin = PortCPin(1 << 0);
pub const PC1: PortCPin = PortCPin(1 << 1);
pub const PC2: PortCPin = PortCPin(1 << 2);
pub const PC3: PortCPin = PortCPin(1 << 3);
pub const PC7: PortCPin = PortCPin(1 << 7);
pub const PC8: PortCPin = PortCPin(1 << 8);
pub const PC9: PortCPin = PortCPin(1 << 9);
pub const PC10: PortCPin = PortCPin(1 << 10);
pub const PC11: PortCPin = PortCPin(1 << 11);
pub const PC12: PortCPin = PortCPin(1 << 12);
pub const PC13: PortCPin = PortCPin(1 << 13); // PROG button on dabao

enum GpioPort {
    PortB = 0,
    PortC = 4,
}

// GPIO register base addresses
//
// Each register is accessed via BASE_ADDRESS + GpioPort offset.
// GpioPort::PortB = 0, GpioPort::PortC = 4
//
// Register    | Port B      | Port C
// ------------|-------------|-------------
// GPIOOUT     | 0x5012f134  | 0x5012f138
// GPIOOE      | 0x5012f14c  | 0x5012f150
// GPIOPU      | 0x5012f164  | 0x5012f168
// GPIOIN      | 0x5012f17c  | 0x5012f180

const GPIOOUT_BASE: *mut u16 = 0x5012f134 as *mut u16;
const GPIOOE_BASE: *mut u16 = 0x5012f14c as *mut u16;
const GPIOPU_BASE: *mut u16 = 0x5012f164 as *mut u16;
const GPIOIN_BASE: *mut u16 = 0x5012f17c as *mut u16;

// Alternate function select registers
const AFSELBL: *mut u16 = 0x5012f008 as *mut u16;
const AFSELBH: *mut u16 = 0x5012f00c as *mut u16;
const AFSELCL: *mut u16 = 0x5012f010 as *mut u16;
const AFSELCH: *mut u16 = 0x5012f014 as *mut u16;

// ============================================================================
// Helper Functions
// ============================================================================

fn register_addr(base: *mut u16, port: GpioPort) -> *mut u16 {
    (base as usize + port as usize) as *mut u16
}

fn gpio_pin_to_parts(pin: GpioPin) -> (GpioPort, u16) {
    match pin {
        GpioPin::PortB(PortBPin(mask)) => (GpioPort::PortB, mask),
        GpioPin::PortC(PortCPin(mask)) => (GpioPort::PortC, mask),
    }
}

fn pin_number_from_mask(mask: u16) -> u8 {
    // Find which bit is set in the mask (assumes only one bit set)
    for i in 0..16 {
        if (mask & (1 << i)) != 0 {
            return i as u8;
        }
    }
    0
}

// ============================================================================
// Public API - GPIO Output Control
// ============================================================================

/// Set pin output high.
///
/// The pin must be configured as an output via `enable_output()` for this
/// to have any effect on the external pin.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn set(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOOUT_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current | mask);
    }
}

/// Set pin output low.
///
/// The pin must be configured as an output via `enable_output()` for this
/// to have any effect on the external pin.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn clear(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOOUT_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current & !mask);
    }
}

/// Toggle pin output state.
///
/// If the pin is high, this sets it low. If it is low, this sets it high.
/// The pin must be configured as an output via `enable_output()` for this
/// to have any effect on the external pin.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn toggle(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOOUT_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current ^ mask);
    }
}

// ============================================================================
// Public API - GPIO Configuration
// ============================================================================

/// Configure pin as an output.
///
/// Sets the output enable bit for this pin. The initial output state is
/// determined by the current GPIOOUT register value.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn enable_output(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOOE_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current | mask);
    }
}

/// Configure pin as an input.
///
/// Clears the output enable bit for this pin. The pin will no longer drive
/// the external signal.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn disable_output(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOOE_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current & !mask);
    }
}

/// Enable internal pull-up on this pin.
///
/// The pull-up is only effective when the pin is configured as an input
/// via `disable_output()`.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn enable_pullup(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOPU_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current | mask);
    }
}

/// Disable internal pull-up on this pin.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn disable_pullup(pin: GpioPin) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOPU_BASE, port);
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current & !mask);
    }
}

/// Read the input state of a pin.
///
/// Returns 1 if the pin is high, 0 if the pin is low. Only meaningful for
/// pins configured as inputs via `disable_output()`.
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
#[inline]
pub fn read_input(pin: GpioPin) -> u16 {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let addr = register_addr(GPIOIN_BASE, port);
        let value = core::ptr::read_volatile(addr);
        if (value & mask) != 0 { 1 } else { 0 }
    }
}

// ============================================================================
// Public API - Alternate Function Selection
// ============================================================================

/// Set the alternate function for a pin.
///
/// Configures which peripheral function (if any) is routed to this pin.
/// AF0 selects GPIO mode (default). AF1, AF2, AF3 select various peripheral
/// functions depending on the pin (UART, I2C, SPI, etc.).
///
/// Example: Configure PB13 and PB14 for UART2:
/// ```ignore
/// set_alternate_function(GpioPin::PortB(PB13), AF::AF1);
/// set_alternate_function(GpioPin::PortB(PB14), AF::AF1);
/// ```
///
/// # Safety
///
/// This function is safe to call because the firmware runs single-threaded.
/// Concurrent GPIO access from multiple threads would cause data races, but
/// that is not possible in this environment.
pub fn set_alternate_function(pin: GpioPin, af: AF) {
    unsafe {
        let (port, mask) = gpio_pin_to_parts(pin);
        let pin_num = pin_number_from_mask(mask);
        let bit_pos = (pin_num % 8) * 2;

        // Determine which AFSEL register to use
        let reg = match port {
            GpioPort::PortB => {
                if pin_num < 8 {
                    AFSELBL
                } else {
                    AFSELBH
                }
            }
            GpioPort::PortC => {
                if pin_num < 8 {
                    AFSELCL
                } else {
                    AFSELCH
                }
            }
        };

        // Clear the 2-bit field for this pin and write new value
        let current = core::ptr::read_volatile(reg);
        let mask_2bit = 0b11u16 << bit_pos;
        let new_val = (current & !mask_2bit) | ((af as u16) << bit_pos);
        core::ptr::write_volatile(reg, new_val);
        // Ensure AF register is set before any GPIO configuration follows
        core::sync::atomic::compiler_fence(
            core::sync::atomic::Ordering::SeqCst,
        );
    }
}
