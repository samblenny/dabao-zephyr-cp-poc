// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//!
//! Bare-metal interrupt handler for Bao1x EventManager-based interrupt system
//!
//! # CSR Access Without External Dependencies
//!
//! This module implements CSR (Control and Status Register) access functions
//! using inline assembly instead of depending on the `riscv` crate. This keeps
//! the core interrupt infrastructure self-contained and zero-dependency.
//!
//! # Hardware Details
//!
//! The Bao1x uses an EventManager-based interrupt system with a tree of
//! interrupts arranged into IRQARRAY banks and some additional CPU-core
//! sources.
//!
//! # Four-Layer Interrupt Enable
//!
//! 1. RISC-V global: mstatus.MIE + mie.MEIP
//! 2. VexRiscv custom: MIM (Machine Interrupt Mask)
//! 3. IRQARRAY event masks: EV_ENABLE bits for individual event sources
//! 4. Peripheral event enable bits (not all peripherals generate interrupts)
//!
//! # Usage
//!
//! ```ignore
//! // Typical user code just sets up peripherals
//! fn callback() {
//!     // This runs in interrupt context
//! }
//! timer0::set_alarm_ms(1000, callback);
//!
//! // (irq_setup() is called automatically at boot via lib.rs::init())
//! ```

use core::arch::asm;
use core::arch::naked_asm;

// ====================================================================
// External Symbols from Linker Script
// ====================================================================

unsafe extern "C" {
    fn _scratch_stack();
}

// ====================================================================
// CSR Register Numbers (Machine Mode)
// ====================================================================

const MSTATUS: u32 = 0x300; // Machine Status
const MIE: u32 = 0x304; // Machine Interrupt Enable
const MTVEC: u32 = 0x305; // Machine Trap Vector
const MCAUSE: u32 = 0x342; // Machine Cause
const MTVAL: u32 = 0x343; // Trap value or fault address
const MIP: u32 = 0x344; // Machine Interrupt Pending flags (RISC-V)
const VEX_MIP: u32 = 0xfc0; // VexRISCV mip (pending interrupt bitfield tree)

// ====================================================================
// Bit Masks for CSRs
// ====================================================================

const MSTATUS_MIE: u32 = 1 << 3; // Global interrupt enable
const MIE_MEIP: u32 = 1 << 11; // Machine external interrupt enable
const MCAUSE_ILLEGAL_INST: u32 = 0x0000_0002; // Illegal instruction exception
const MCAUSE_LOAD_ACCESS: u32 = 0x0000_0005; // Memory load caused fault
const MCAUSE_EXTERNAL_INT: u32 = 0x8000_000B; // External interrupt code

// ====================================================================
// Bit Masks for VexRISCV MIP (pending interrupt event bitfield)
// ====================================================================

const VEX_MIP_TIMER0_BIT: u32 = 1 << 30; // TIMER0 alarm event bit

// ====================================================================
// MIM Register Bit Masks (Machine Interrupt Mask - enable IRQARRAY banks)
// ====================================================================

// const MIM_BIT_TICKTIMER: u32 = 1 << 20;
const MIM_BIT_TIMER0: u32 = 1 << 30;

// ====================================================================
// CSR Helper Functions (No External Dependencies)
// ====================================================================

/// Read a CSR register by number
#[inline]
fn csr_read(csr: u32) -> u32 {
    let result: u32;
    unsafe {
        match csr {
            MSTATUS => asm!("csrr {0}, mstatus", out(reg) result),
            MIE => asm!("csrr {0}, mie", out(reg) result),
            MTVEC => asm!("csrr {0}, mtvec", out(reg) result),
            MTVAL => asm!("csrr {0}, mtval", out(reg) result),
            MCAUSE => asm!("csrr {0}, mcause", out(reg) result),
            MIP => asm!("csrr {0}, mip", out(reg) result),
            VEX_MIP => asm!("csrr {0}, 0xfc0", out(reg) result),
            _ => result = 0, // Unsupported CSR
        }
    }
    result
}

/// Write a CSR register by number
#[inline]
fn csr_write(csr: u32, value: u32) {
    unsafe {
        match csr {
            MTVEC => asm!("csrw mtvec, {0}", in(reg) value),
            MSTATUS => asm!("csrw mstatus, {0}", in(reg) value),
            MIE => asm!("csrw mie, {0}", in(reg) value),
            _ => {}
        }
    }
}

/// Set bits in a CSR register (CSR |= value)
#[inline]
fn csr_set(csr: u32, bits: u32) {
    unsafe {
        match csr {
            MSTATUS => asm!("csrs mstatus, {0}", in(reg) bits),
            MIE => asm!("csrs mie, {0}", in(reg) bits),
            _ => {}
        }
    }
}

/// Clear bits in a CSR register (CSR &= ~value)
#[inline]
fn csr_clear(csr: u32, bits: u32) {
    unsafe {
        match csr {
            MIE => asm!("csrc mie, {0}", in(reg) bits),
            MSTATUS => asm!("csrc mstatus, {0}", in(reg) bits),
            _ => {}
        }
    }
}

/// Write VexRiscv custom MIM (Machine Interrupt Mask) register (0xBC0).
///
/// MIM is not a standard RISC-V CSR. It is specific to VexRiscv and controls
/// which IRQARRAY banks can generate interrupts to the CPU.
#[inline]
fn csr_write_mim(value: u32) {
    unsafe {
        asm!("csrw 0xbc0, {0}", in(reg) value);
    }
}

/// Set bits in VexRiscv custom MIM (Machine Interrupt Mask) register (0xBC0).
///
/// MIM is not a standard RISC-V CSR. It is specific to VexRiscv and controls
/// which IRQARRAY banks can generate interrupts to the CPU.
#[inline]
fn csr_set_mim(bits: u32) {
    unsafe {
        asm!("csrs 0xbc0, {0}", in(reg) bits);
    }
}

// ====================================================================
// Public Interrupt Setup Functions
// ====================================================================

/// Initialize trap handler and enable interrupts
///
/// Must be called once at boot before any interrupts are enabled.
/// Disables all peripheral interrupt sources, sets up mtvec, clears MIM, and
/// enables global interrupt bits.
pub fn irq_setup() {
    // Get trap handler address (from linker script)
    let handler_addr = _trap as *const () as u32;

    // Store trap handler address in mtvec. Note that _trap is aligned to
    // 16-bytes by the linker script, so bits [1:0] are clear (as needed for
    // direct addressing mode).
    csr_write(MTVEC, handler_addr);

    // Ensure trap handler is configured before enabling interrupts
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    // Initially disable the full tree of interrupt sources at the top level
    csr_write_mim(0);

    // Ensure MIM is configured before enabling global interrupts
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    // Enable global machine interrupt enable (mstatus.MIE)
    csr_set(MSTATUS, MSTATUS_MIE);

    // Enable machine external interrupts (mie.MEIP)
    csr_set(MIE, MIE_MEIP);

    // Enable TIMER0 events
    csr_set_mim(MIM_BIT_TIMER0);
}

/// Enable all interrupts
#[inline]
pub fn enable_irqs() {
    // Enable global machine interrupt enable (mstatus.MIE)
    csr_set(MSTATUS, MSTATUS_MIE);
}

/// Disable all interrupts, returning previous enable status
#[inline]
pub fn disable_irqs() -> bool {
    // Check if interrupts were already disabled
    let was_enabled = csr_read(MSTATUS) & MSTATUS_MIE != 0;

    // Clear global interrupt enable
    csr_clear(MSTATUS, MSTATUS_MIE);

    was_enabled
}

// ====================================================================
// Trap Handler Assembly Entry Point
// ====================================================================

/// Trap handler entry (assembly)
///
/// Saves all registers to scratch page and jumps to Rust dispatcher.
///
/// # Important: Alignment Requirement
///
/// VexRiscv requires _trap to be 4-byte aligned. The linker script
/// (link.x) provides this alignment via the .text._trap section.
/// Do not modify this function's alignment without updating link.x.
#[unsafe(export_name = "_trap")]
#[unsafe(naked)]
pub unsafe extern "C" fn _trap() -> ! {
    naked_asm!(
        // Disable nested interrupts: save mstatus and Clear MIE (bit 3)
        "csrrc t1, mstatus, 1 << 3",

        // Save original SP to mscratch
        "csrw   mscratch, sp",

        // Set SP to scratch page
        "la     sp, {0}", // sym _scratch_stack

        // Allocate space for registers leaving sp aligned to 16 bytes
        "addi sp, sp, -(36*4)",

        // Save all general-purpose registers (x1-x31)
        "sw     x1,  0*4(sp)",   // ra
        // Skip x2 (sp) for now
        "sw     x3,  2*4(sp)",   // gp
        "sw     x4,  3*4(sp)",   // tp
        "sw     x5,  4*4(sp)",   // t0
        "sw     x6,  5*4(sp)",   // t1
        "sw     x7,  6*4(sp)",   // t2
        "sw     x8,  7*4(sp)",   // s0
        "sw     x9,  8*4(sp)",   // s1
        "sw     x10, 9*4(sp)",   // a0
        "sw     x11, 10*4(sp)",  // a1
        "sw     x12, 11*4(sp)",  // a2
        "sw     x13, 12*4(sp)",  // a3
        "sw     x14, 13*4(sp)",  // a4
        "sw     x15, 14*4(sp)",  // a5
        "sw     x16, 15*4(sp)",  // a6
        "sw     x17, 16*4(sp)",  // a7
        "sw     x18, 17*4(sp)",  // s2
        "sw     x19, 18*4(sp)",  // s3
        "sw     x20, 19*4(sp)",  // s4
        "sw     x21, 20*4(sp)",  // s5
        "sw     x22, 21*4(sp)",  // s6
        "sw     x23, 22*4(sp)",  // s7
        "sw     x24, 23*4(sp)",  // s8
        "sw     x25, 24*4(sp)",  // s9
        "sw     x26, 25*4(sp)",  // s10
        "sw     x27, 26*4(sp)",  // s11
        "sw     x28, 27*4(sp)",  // t3
        "sw     x29, 28*4(sp)",  // t4
        "sw     x30, 29*4(sp)",  // t5
        "sw     x31, 30*4(sp)",  // t6

        // Save mepc
        "csrr   t0, mepc",
        "sw     t0, 31*4(sp)",

        // Save mstatus
        "sw     t1, 32*4(sp)",

        // Save original SP (from mscratch)
        "csrr   t0, mscratch",
        "sw     t0, 1*4(sp)",

        // =========================================
        // Call to Rust: Dispatch interrupt handlers
        // =========================================

        "call   {1}", // sym _trap_handler_rust

        // ========================================================
        // Exit Routine: Restore all registers and return from trap
        // ========================================================

        // Set SP to scratch page
        "la     sp, {0}", // sym _scratch_stack

        // Adjust sp to match the trap frame allocation in the entry routine
        // CAUTION: This assumes nested traps are not allowed
        "addi sp, sp, -(36*4)",

        // Load all general-purpose registers
        "lw     x1,  0*4(sp)",  // ra
        // skip x2 (sp) for now
        "lw     x3,  2*4(sp)",  // gp
        "lw     x4,  3*4(sp)",  // tp
        "lw     x5,  4*4(sp)",  // t0
        "lw     x6,  5*4(sp)",  // t1
        "lw     x7,  6*4(sp)",  // t2
        "lw     x8,  7*4(sp)",  // s0
        "lw     x9,  8*4(sp)",  // s1
        "lw     x10, 9*4(sp)",  // a0
        "lw     x11, 10*4(sp)", // a1
        "lw     x12, 11*4(sp)", // a2
        "lw     x13, 12*4(sp)", // a3
        "lw     x14, 13*4(sp)", // a4
        "lw     x15, 14*4(sp)", // a5
        "lw     x16, 15*4(sp)", // a6
        "lw     x17, 16*4(sp)", // a7
        "lw     x18, 17*4(sp)", // s2
        "lw     x19, 18*4(sp)", // s3
        "lw     x20, 19*4(sp)", // s4
        "lw     x21, 20*4(sp)", // s5
        "lw     x22, 21*4(sp)", // s6
        "lw     x23, 22*4(sp)", // s7
        "lw     x24, 23*4(sp)", // s8
        "lw     x25, 24*4(sp)", // s9
        "lw     x26, 25*4(sp)", // s10
        "lw     x27, 26*4(sp)", // s11
        "lw     x28, 27*4(sp)", // t3
        "lw     x29, 28*4(sp)", // t4
        "lw     x30, 29*4(sp)", // t5
        "lw     x31, 30*4(sp)", // t6

        // Load mepc
        "lw     t0, 31*4(sp)",
        "csrw   mepc, t0",

        // Load mstatus (at this point MIE is probably set)
        "lw     t0, 32*4(sp)",

        // Ensure MIE bit is clear (mret will turn it on after we restorw sp)
        "andi   t0, t0, ~(1 << 3)",
        "csrw   mstatus, t0",

        // Load original SP (CAUTION: this must come last)
        "lw     x2, 1*4(sp)",

        // Return from trap (restores PC from mepc)
        "mret",

        // Rust docs say it's bad to use named labels like {foo}. Instead,
        // we're supposed to use numeric labels like {0} or {1}. See:
        // https://doc.rust-lang.org/rust-by-example/unsafe/asm.html#labels
        sym _scratch_stack,
        sym _trap_handler_rust,
    );
}

// ====================================================================
// Rust Trap Dispatcher
// ====================================================================

/// Rust-level trap handler dispatcher
///
/// Reads mcause to determine interrupt type, checks IRQARRAY0 pending
/// events, and dispatches to appropriate handler.
pub extern "C" fn _trap_handler_rust() {
    // Debug: Turn on LED at PB12 to indicate trap was hit
    crate::gpio::set_alternate_function(
        crate::gpio::GpioPin::PortB(crate::gpio::PB12),
        crate::gpio::AF::AF0,
    );
    crate::gpio::enable_output(crate::gpio::GpioPin::PortB(crate::gpio::PB12));
    crate::gpio::set(crate::gpio::GpioPin::PortB(crate::gpio::PB12));

    // Read mcause and mip for dispatch
    let mcause = csr_read(MCAUSE);

    // Check if this is an external interrupt
    if mcause == MCAUSE_EXTERNAL_INT {
        // CRITICAL: VexRISCV overloads the normal RISC-V CSR named "mip" with
        // a different meaning and a different CSR number. It's very confusing!
        // At https://docs.riscv.org/reference/isa/priv/priv-csrs.html, mip is
        // listed as CSR number 0x343 in the Machine Trap Handling section. At
        // xous-core/imports/riscv-0.5.6/src/register/vexriscv/mip.rs on the
        // bettrusted-io GitHub, mip is defined as CSR 0xFC0. And, if you look
        // at the `let irqs_pending = mip::read();` usage in
        // xous-core/baremetal/src/platform/bao1x/irq.rs, you can see that it's
        // used as a bitfield corresponding to the assigned interrupt numbers
        // listed at https://ci.betrusted.io/bao1x-cpu/interrupts.html

        let pending = csr_read(VEX_MIP);

        // Check for TIMER0 event
        if pending & VEX_MIP_TIMER0_BIT != 0 {
            timer0_handler();
        } else {
            // Add more event checks here as needed (UART, USB, etc.)
            crate::log!("  TRAP: external vex_mip=0x{:08x}\r\n", pending);
            crate::sleep(2);
        }
    } else if mcause == MCAUSE_ILLEGAL_INST {
        crate::log!("\r\nTRAP: illegal instruction\r\n");
        crate::sleep(2);
        loop {}
    } else if mcause == MCAUSE_LOAD_ACCESS {
        let mtval = csr_read(MTVAL);
        crate::log!("\r\nTRAP: load access, mtval=0x{:08x}", mtval);
        crate::sleep(2);
        loop {}
    } else {
        // Unknown exception
        crate::log!("\r\nTRAP: mcause=0x{:08x}\r\n", mcause);
        crate::sleep(2);
        loop {}
    }

    // Turn off LED before returning
    crate::gpio::clear(crate::gpio::GpioPin::PortB(crate::gpio::PB12));
}

// ====================================================================
// TIMER0 Interrupt Handler
// ====================================================================

/// Handle TIMER0 interrupt
///
/// Called from trap dispatcher when TIMER0 fires.
/// Clears pending bit to allow next interrupt.
#[inline]
fn timer0_handler() {
    // Clear pending bit and ensure timer won't accidentally re-trigger
    crate::timer0::stop_and_clear();

    // Invoke callback if registered
    if let Some(callback) = crate::timer0::get_callback() {
        callback();
    }
}
