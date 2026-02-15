#include "baochip_sdk.h"
#include <stdio.h>

// Example C code using baochip_sdk Rust drivers.
// This demonstrates how C code can call Rust driver functions.
//
// Key constraint: main() must never return. The Rust lib.rs _start()
// entry point calls this main() after hardware initialization, and
// expects it to loop forever.
//
int main(void) {
    char buf[128];

    // Infinite loop: main() must never return.
    // Each iteration: format a string, send via UART, sleep 5 seconds.
    for (int i = 0; ; i = (i + 1) & 0xff) {
        size_t n;
        n = snprintf(buf, sizeof(buf), "Hello, world! (from C; i=%d)\r\n", i);
        // snprintf returns the length of the string it would write to an
        // unlimited buffer, even if it actually truncated the string to fit
        // the real buffer. So, the length technically might be too long.
        if (n > sizeof(buf)) {
            n = sizeof(buf);
        }
        dbs_uart_write((const uint8_t *)buf, n);
        dbs_timer_sleep_ms(5000);
    }
}
