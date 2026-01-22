.PHONY: blinky blinky-disassemble blinky-bin-hex blinky-img-hex blinky-uf2-hex
.PHONY: clean

STABLE_LIB := $(HOME)/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib
LLVM_BIN := $(STABLE_LIB)/rustlib/x86_64-unknown-linux-gnu/bin
TARGET_DIR := target/riscv32imac-unknown-none-elf/debug/examples
BLINKY := $(TARGET_DIR)/blinky

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
	python3 signer.py $(BLINKY).bin $(BLINKY).img
	@echo '---'
	@echo '# Packing signed blob as UF2:'
	python3 uf2ify.py $(BLINKY).img $(BLINKY).uf2

blinky-disassemble:
	$(LLVM_BIN)/llvm-objdump -d $(BLINKY) | less

blinky-bin-hex:
	hexdump -C $(BLINKY).bin | less

blinky-img-hex:
	hexdump -C $(BLINKY).img | less

blinky-uf2-hex:
	hexdump -C $(BLINKY).uf2 | less

clean:
	cargo clean
