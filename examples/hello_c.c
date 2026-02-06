#include "dabao_sdk.h"
#include <stdio.h>

int c_main() {
    char buf[128];
    for (int i = 0; 1; i = (i + 1) & 0xff) {
        sprintf(buf, "Hello, world! (from C; i=%d)\r\n", i);
        uart_write((const uint8_t *)buf);
        sleep(5000);
    }
}
