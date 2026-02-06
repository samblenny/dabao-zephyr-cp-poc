#ifndef DABAO_SDK_H
#define DABAO_SDK_H

#include <stdint.h>

void uart_write(const uint8_t *data);
void tick();
void sleep(uint32_t ms);

#endif // DABAO_SDK_H
