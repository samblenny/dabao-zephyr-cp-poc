#include "dabao_sdk.h"
#include <stdio.h>

// Example C code using dabao_sdk Rust drivers.
// This demonstrates how C code can call Rust driver functions.
//
// Key constraint: main() must never return. The Rust lib.rs _start()
// entry point calls this main() after hardware initialization, and
// expects it to loop forever.
//
// The Makefile compiles this C code to a static library, then links
// it with c_main_wrapper.rs (which declares the extern main()) using
// RUSTFLAGS to point the linker at the C library.
int main() {
    char buf[128];

    // Infinite loop: main() must never return.
    // Each iteration: format a string, send via UART, sleep 5 seconds.
    for (int i = 0; ; i = (i + 1) & 0xff) {
        sprintf(buf, "Hello, world! (from C; i=%d)\r\n", i);
        uart_write((const uint8_t *)buf);
        sleep(5000);
    }
}
