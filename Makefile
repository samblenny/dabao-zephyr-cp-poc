.PHONY: blinky blinky-disassemble blinky-bin-hex blinky-img-hex blinky-uf2-hex
.PHONY: uart uart-disassemble uart-bin-hex uart-img-hex uart-uf2-hex
.PHONY: timer0 timer0-disassemble timer0-bin-hex timer0-img-hex timer0-uf2-hex
.PHONY: hello_c
.PHONY: clean

STABLE_LIB := $(HOME)/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib
LLVM_BIN := $(STABLE_LIB)/rustlib/x86_64-unknown-linux-gnu/bin
TARGET_DIR := target/riscv32imac-unknown-none-elf/debug
BLINKY := $(TARGET_DIR)/examples/blinky
UART := $(TARGET_DIR)/examples/uart
TIMER0 := $(TARGET_DIR)/examples/timer0

# Picolibc include and lib paths
LIBC_DIR := /usr/lib/picolibc/riscv64-unknown-elf/lib/release/rv32imac/ilp32
CFLAGS := -I/usr/lib/picolibc/riscv64-unknown-elf/include \
	-march=rv32imac -mabi=ilp32


hello_c:
	cargo clean
	mkdir -p $(TARGET_DIR)/examples
	@echo '---'
	@echo "# Compiling C code..."
	riscv64-unknown-elf-gcc $(CFLAGS) -c examples/hello_c.c \
		-o $(TARGET_DIR)/examples/hello_c.o
	@echo '---'
	@echo "# Archiving C library..."
	riscv64-unknown-elf-ar rcs $(TARGET_DIR)/examples/libhello_c.a \
		$(TARGET_DIR)/examples/hello_c.o
	@echo '---'
	@echo "# Building Rust wrapper..."
	RUSTFLAGS="-l hello_c -l c \
		-L target/riscv32imac-unknown-none-elf/debug/examples \
		-L $(LIBC_DIR) \
		-C panic=abort \
		-C opt-level=s \
		-C debuginfo=none \
		-C link-arg=-Tlink.x" \
		cargo build --example c_main_wrapper \
		--target riscv32imac-unknown-none-elf
	@echo '---'
	$(LLVM_BIN)/llvm-objcopy -O binary $(TARGET_DIR)/examples/c_main_wrapper \
		hello_c.bin
	@echo '---'
	python3 signer.py hello_c.bin hello_c.img
	@echo '---'
	python3 uf2ify.py hello_c.img hello_c.uf2
	@echo '---'
	cp hello_c.uf2 examples/

# Rebuild from scratch every time to avoid the hassle of defining the tree
# of dependencies between sources and outputs.
blinky:
	cargo clean
	cargo build --example blinky
	objdump -h $(BLINKY)
	@echo '---'
	@echo '# Checking .data section LMA (FLASH) and VMA (RAM) addresses:'
	@echo 'llvm-objdump -t blinky | grep _data'
	@$(LLVM_BIN)/llvm-objdump -t $(BLINKY) | grep _data
	@echo '---'
	@echo '# Extracting loadable sections to .bin file:'
	@echo 'llvm-objcopy -O binary blinky blinky.bin'
	@$(LLVM_BIN)/llvm-objcopy -O binary $(BLINKY) $(BLINKY).bin
	@echo '---'
	@echo '# Signing .bin file:'
	@python3 signer.py $(BLINKY).bin $(BLINKY).img
	@echo '---'
	@echo '# Packing signed blob as UF2:'
	@python3 uf2ify.py $(BLINKY).img $(BLINKY).uf2
	@echo '---'
	cp $(BLINKY).uf2 examples/

blinky-disassemble:
	$(LLVM_BIN)/llvm-objdump -d $(BLINKY) | less

blinky-bin-hex:
	hexdump -C $(BLINKY).bin | less

blinky-img-hex:
	hexdump -C $(BLINKY).img | less

blinky-uf2-hex:
	hexdump -C $(BLINKY).uf2 | less

uart:
	cargo clean
	cargo build --example uart
	objdump -h $(UART)
	@echo '---'
	@echo '# Checking .data section LMA (FLASH) and VMA (RAM) addresses:'
	@echo 'llvm-objdump -t uart | grep _data'
	@$(LLVM_BIN)/llvm-objdump -t $(UART) | grep _data
	@echo '---'
	@echo '# Extracting loadable sections to .bin file:'
	@echo 'llvm-objcopy -O binary uart uart.bin'
	@$(LLVM_BIN)/llvm-objcopy -O binary $(UART) $(UART).bin
	@echo '---'
	@echo '# Signing .bin file:'
	@python3 signer.py $(UART).bin $(UART).img
	@echo '---'
	@echo '# Packing signed blob as UF2:'
	@python3 uf2ify.py $(UART).img $(UART).uf2
	@echo '---'
	cp $(UART).uf2 examples/

uart-disassemble:
	$(LLVM_BIN)/llvm-objdump -d $(UART) | less

uart-bin-hex:
	hexdump -C $(UART).bin | less

uart-img-hex:
	hexdump -C $(UART).img | less

uart-uf2-hex:
	hexdump -C $(UART).uf2 | less

timer0:
	cargo clean
	cargo build --example timer0
	objdump -h $(TIMER0)
	@echo '---'
	@echo '# Checking .data section LMA (FLASH) and VMA (RAM) addresses:'
	@echo 'llvm-objdump -t timer0 | grep _data'
	@$(LLVM_BIN)/llvm-objdump -t $(TIMER0) | grep _data
	@echo '---'
	@echo '# Extracting loadable sections to .bin file:'
	@echo 'llvm-objcopy -O binary timer0 timer0.bin'
	@$(LLVM_BIN)/llvm-objcopy -O binary $(TIMER0) $(TIMER0).bin
	@echo '---'
	@echo '# Signing .bin file:'
	@python3 signer.py $(TIMER0).bin $(TIMER0).img
	@echo '---'
	@echo '# Packing signed blob as UF2:'
	@python3 uf2ify.py $(TIMER0).img $(TIMER0).uf2
	@echo '---'
	cp $(TIMER0).uf2 examples/

timer0-disassemble:
	$(LLVM_BIN)/llvm-objdump -d $(TIMER0) | less

timer0-bin-hex:
	hexdump -C $(TIMER0).bin | less

timer0-img-hex:
	hexdump -C $(TIMER0).img | less

timer0-uf2-hex:
	hexdump -C $(TIMER0).uf2 | less

clean:
	cargo clean
