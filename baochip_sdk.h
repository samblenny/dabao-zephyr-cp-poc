// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
//
// dabao-sdk C FFI interface
//
// This header provides C-compatible function declarations for the dabao-sdk
// Rust driver library. All functions use the dbs_ prefix (dabao-sdk).
//
// The functions are implemented in src/lib.rs as extern "C" functions that
// adapt Rust idioms to C idioms.

#ifndef BAOCHIP_SDK_H
#define BAOCHIP_SDK_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// UART Functions
// ============================================================================

/// Read one character from UART2, blocking until available.
///
/// This function blocks (with sleep calls) until a character is available
/// from the UART2 receive buffer. The sleep() call ensures that the
/// transmit DMA queue is serviced while waiting.
///
/// @return The character read as an unsigned byte.
uint8_t dbs_uart_read_char(void);

/// Write data to UART2.
///
/// Queues the data for transmission via DMA. The write is non-blocking;
/// the function returns immediately. Call dbs_uart_tick() periodically
/// to service the TX DMA queue.
///
/// @param data Pointer to bytes to write
/// @param len Number of bytes to write
void dbs_uart_write(const uint8_t *data, size_t len);

/// Service UART2 transmit DMA queue.
///
/// Checks if the current DMA transfer is complete. If so, advances the
/// queue and starts the next DMA transfer if available.
///
/// Call periodically from your main event loop. Also called automatically
/// by dbs_uart_write() when needed.
void dbs_uart_tick(void);

// ============================================================================
// Timer Functions
// ============================================================================

/// Sleep for specified milliseconds.
///
/// Blocks until the specified time has elapsed. Calls dbs_uart_tick()
/// periodically to ensure UART transmit queue is serviced.
///
/// @param ms Number of milliseconds to sleep
void dbs_timer_sleep_ms(uint32_t ms);

/// Get current system time in milliseconds.
///
/// Returns the number of milliseconds elapsed since system boot.
///
/// @return Current time in milliseconds
uint64_t dbs_timer_millis(void);

// ============================================================================
// GPIO Functions (placeholder - not yet implemented)
// ============================================================================

// GPIO functions will be added in Phase 3.5
// Examples:
//   dbs_gpio_set_level(pin, level)
//   dbs_gpio_get_level(pin)
//   dbs_gpio_set_dir(pin, direction)

#ifdef __cplusplus
}
#endif

#endif // BAOCHIP_SDK_H
