// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
//! USB support for bao1x dabao evaluation board
//!
//! This module implements USB device functionality using the Corigine USB
//! controller. Implementation follows a phased approach starting with basic
//! hardware detection and progressing toward full CDC-ACM serial support.

use core::ptr;

// ============================================================================
// IRQARRAY1 Register Addresses
// ============================================================================

const IRQARRAY1_EV_SOFT: *mut u32 = 0xe0005000 as *mut u32;
//const IRQARRAY1_EV_EDGE_TRIGGERED: *mut u32 = 0xe0005004 as *mut u32;
//const IRQARRAY1_EV_POLARITY: *mut u32 = 0xe0005008 as *mut u32;
//const IRQARRAY1_EV_STATUS: *const u32 = 0xe000500c as *const u32;
const IRQARRAY1_EV_PENDING: *mut u32 = 0xe0005010 as *mut u32;
//const IRQARRAY1_EV_ENABLE: *mut u32 = 0xe0005014 as *mut u32;

// Bit mask for USB controller in IRQARRAY1
const USBC_BIT: u32 = 1 << 0;

// ============================================================================
// Corigine USB Controller Register Addresses
// ============================================================================

const CORIGINE_BASE: u32 = 0x5020_2400;

// Device register offsets from Corigine base
const REG_DEVCAP: u32 = 0x0000;
//const REG_DEVCONFIG: u32 = 0x0010;
//const REG_USBCMD: u32 = 0x0020;
//const REG_USBSTS: u32 = 0x0024;

// ============================================================================
// Phase 0: IRQARRAY1_EV_PENDING Writability Test (CONFIRMED)
// ============================================================================

/// Phase 0 validation: IRQARRAY1_EV_PENDING uses RW1C semantics.
///
/// TESTED AND CONFIRMED: Writing 1 to a pending bit clears it to 0.
/// This function was executed on hardware and verified to work correctly.
/// USB interrupt handlers can safely clear pending flags by writing 1.
///
/// For reference, the test:
/// 1. Uses EV_SOFT to set the pending bit
/// 2. Writes 1 to attempt clearing via RW1C
/// 3. Verifies the bit was cleared
/// 4. Logs detailed results for debugging
#[allow(dead_code)]
pub fn pending_write_test() {
    unsafe {
        crate::log!("IRQARRAY1 Writability Test\r\n");

        // Use software trigger to set the pending bit
        crate::log!("  Setting USBC_BIT via EV_SOFT...\r\n");
        ptr::write_volatile(IRQARRAY1_EV_SOFT, USBC_BIT);
        crate::sleep(1);

        // Read pending to verify bit is set
        let pending_after_set = ptr::read_volatile(IRQARRAY1_EV_PENDING);
        crate::log!(
            "  After EV_SOFT write, EV_PENDING = 0x{:08x}\r\n",
            pending_after_set
        );

        if (pending_after_set & USBC_BIT) == 0 {
            crate::log!("  ERROR: EV_SOFT didn't set pending bit!\r\n");
            return;
        }

        // Attempt to clear by writing 1 (test RW1C semantics)
        crate::log!("  Attempting to clear by writing 1 to bit...\r\n");
        ptr::write_volatile(IRQARRAY1_EV_PENDING, USBC_BIT);
        crate::sleep(1);

        // Check whether the bit was cleared
        let pending_after_clear = ptr::read_volatile(IRQARRAY1_EV_PENDING);
        crate::log!(
            "  After write, EV_PENDING = 0x{:08x}\r\n",
            pending_after_clear
        );

        // Report results
        if (pending_after_clear & USBC_BIT) == 0 {
            crate::log!("  SUCCESS: Bit was cleared by write\r\n");
        } else {
            crate::log!("  WARNING: Bit was NOT cleared by write\r\n");
            crate::log!(
                "  Clearing may require different mechanism (e.g., peripheral action)\r\n"
            );
        }

        // Clean up state
        crate::log!("  Clearing EV_SOFT...\r\n");
        ptr::write_volatile(IRQARRAY1_EV_SOFT, 0);
        crate::sleep(1);

        let final_pending = ptr::read_volatile(IRQARRAY1_EV_PENDING);
        crate::log!("  Final EV_PENDING = 0x{:08x}\r\n", final_pending);
    }
}

// ============================================================================
// Phase 1: USB Controller Detection (Stub)
// ============================================================================

/// Detect if USB controller is present and accessible.
///
/// Reads DEVCAP register and validates device capabilities.
/// Returns true if controller responds with valid version/features.
///
/// # Currently:
/// This is a placeholder for Phase 1 implementation.
pub fn detect() -> bool {
    unsafe {
        let devcap =
            ptr::read_volatile((CORIGINE_BASE + REG_DEVCAP) as *const u32);
        crate::log!("USB DEVCAP = 0x{:08x}\r\n", devcap);
        // TODO: Validate DEVCAP version and features
        devcap != 0xffffffff // Basic sanity check
    }
}

// ============================================================================
// Phase 2: Minimal Enumeration Setup (Stub)
// ============================================================================

/// Initialize USB controller for basic enumeration.
///
/// Sets up Device Context, Event Ring, and EP0 configuration.
///
/// # Currently:
/// This is a placeholder for Phase 2 implementation.
pub fn init() {
    crate::log!("USB init (stub)\r\n");
    // TODO: Phase 2 implementation
}

// ============================================================================
// Phase 3: Interrupt Handler (Stub)
// ============================================================================

/// Handle USB interrupt from IRQARRAY1.
///
/// Called from trap handler when IRQARRAY1_EV_PENDING bit 0 fires.
///
/// # Currently:
/// This is a placeholder for Phase 3 implementation.
pub fn handle_interrupt() {
    // TODO: Phase 3 implementation
}
