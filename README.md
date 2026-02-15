<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: Copyright 2026 Sam Blenny -->
# Dabao SDK

**DRAFT: WORK IN PROGRESS**

[*an aspirational blurb about what this is trying to be*:]

This is a baremetal SDK for the Baochip Dabao Bao1x evaluation board. Most of
the work here is about peripheral drivers, linker scripts, firmware blob
signing, and examples to demonstrate driver usage. My goal is to write the
drivers in no_std Rust for safety and make the top API layer callable from
either Rust or C for compatibility.

Backstory: This originated as an attempt to do a Zephyr + CircuitPython port
for Dabao. But, I changed my mind and scoped this down to just making an SDK
with examples and docs.

**CAUTION:** This is a proof of concept. I don't currently plan to maintain
this code once I've got it working. The point is to document how to do it.


## Roadmap

1. Implement Rust drivers and usage examples for timers, GPIO, UART, and USB
   CDC-ACM serial.

2. Add a C FFI layer over the drivers and port the examples to C. This will be
   mostly about Makefile and linker script tricks to get a rust library working
   with C application code.

3. Figure out how to build rv32e machine code blobs to run on the BIO cores and
   how to get them working with rv32imac code for the main CPU. This will
   involve more Makefile and linker script tricks.

4. Do a documentation pass to make sure all the above are explained clearly.

5. *Maybe:* Come back around for a USB CDC-NCM and UDP/IP stack pass. The Bao1x
   has a relatively fast FS/HS USB controller but it only supports 4 endpoints.
   Assuming 2 endpoints will always be used for CDC-ACM serial, that doesn't
   leave much room for other modes of IO. Using the second pair of endpoints
   for a virtual ethernet link would allow muxing whatever other types of IO
   you might want over IP.


## Dev Environment Setup

These instructions are for Debian 13 (Trixie), but they should probably work
with minimal modifications on whatever the latest Ubuntu LTS release is.
You're on your own for adapting to other Linux flavors, Windows, or macOS.

1. Install Rust using the rustup install procedure at:
   [rust-lang.org/learn/get-started/](https://rust-lang.org/learn/get-started/)

   If you're allergic to piping curl into a shell, download the script on your
   own first and give it a read before you run it. You might also consider
   selecting the script's "Customize installation" option and telling it not to
   automatically modify your PATH, then modify PATH manually yourself.

2. Install the riscv32imac-unknown-none-elf target with rustup:

   ```
   rustup target add riscv32imac-unknown-none-elf
   ```

3. Install gcc rv32 compiler, binutils, and picolibc:

   ```
   sudo apt install binutils-riscv64-unknown-elf \
     gcc-riscv64-unknown-elf \
     picolibc-riscv64-unknown-elf
   ```

   Picolibc includes are at `/usr/lib/picolibc/riscv64-unknown-elf/include`,
   and `/usr/lib/picolibc/riscv64-unknown-elf/lib/release/rv32imac/ilp32` has
   the release builds of the libraries. This is for Debian. YMMV for Ubuntu.


## Building the Examples

This uses a Makefile to orchestrate `cargo build` along with some llvm tools
for post build binary manipulation and a couple Python scripts to sign and UF2
pack the firmware blob.


### examples/hello_c.c (C linked to Rust)

This one uses C for the application logic (hello world) and Rust for the
hardware drivers. The C code has a hello world that gets linked with picolibc
and a Rust wrapper. The Rust code initializes the hardware, provides a UART
driver, and calls the `main()` exported by the C library. The C code is at
[examples/hello_c.c](examples/hello_c.c).

```
$ make hello_c
cargo clean
     Removed 43 files, 17.7MiB total
mkdir -p target/riscv32imac-unknown-none-elf/debug/examples
---
# Compiling C code...
riscv64-unknown-elf-gcc \
	-I/usr/lib/picolibc/riscv64-unknown-elf/include -march=rv32imac -mabi=ilp32 \
	-c examples/hello_c.c \
	-o target/riscv32imac-unknown-none-elf/debug/examples/hello_c.o
---
# Archiving C library...
riscv64-unknown-elf-ar rcs \
	target/riscv32imac-unknown-none-elf/debug/examples/libhello_c.a \
	target/riscv32imac-unknown-none-elf/debug/examples/hello_c.o
---
# Building Rust SDK library (libdabao_sdk.a)...
cargo build --lib
   Compiling dabao-sdk v0.1.0 (/home/sam/code/dabao-sdk)
    Finished `dev` profile [optimized] target(s) in 0.18s
---
# Linking C library with Rust library...
riscv64-unknown-elf-gcc \
	-march=rv32imac -mabi=ilp32 -nostartfiles -nostdlib \
	-Tlink.x \
	-Wl,--gc-sections \
	-o target/riscv32imac-unknown-none-elf/debug/examples/hello_c.elf \
	target/riscv32imac-unknown-none-elf/debug/libdabao_sdk.a \
	target/riscv32imac-unknown-none-elf/debug/examples/libhello_c.a \
	/usr/lib/picolibc/riscv64-unknown-elf/lib/release/rv32imac/ilp32/libc.a \
	-lgcc
---
# Extracting loadable sections to .bin file:
llvm-objcopy -O binary hello_c.elf hello_c.bin
---
# Signing .bin file:
binary payload size is 22040 bytes
Signed firmware blob written to target/riscv32imac-unknown-none-elf/debug/examples/hello_c.img
---
# Packing signed blob as UF2:
signed blob file size is 22808 bytes
UF2 image written to target/riscv32imac-unknown-none-elf/debug/examples/hello_c.uf2
---
cp target/riscv32imac-unknown-none-elf/debug/examples/hello_c.uf2 examples/
```


### examples/blinky.rs

See comments in [examples/blinky.rs](examples/blinky.rs) for more details on
what this does.

```
$ make blinky
cargo clean
     Removed 54 files, 9.7MiB total
cargo build --example blinky
   Compiling dabao-sdk v0.1.0 (/home/sam/code/dabao-sdk)
    Finished `dev` profile [optimized] target(s) in 0.24s
objdump -h target/riscv32imac-unknown-none-elf/debug/examples/blinky

target/riscv32imac-unknown-none-elf/debug/examples/blinky:     file format elf32-little

Sections:
Idx Name          Size      VMA       LMA       File off  Algn
  0 .firmware     00001cf0  60060300  60060300  00000300  2**2
                  CONTENTS, ALLOC, LOAD, READONLY, CODE
  1 .data         00000010  61000000  60061ff0  00002000  2**2
                  CONTENTS, ALLOC, LOAD, DATA
  2 .bss          00000020  61000010  61000010  00002010  2**2
                  ALLOC
---
# Checking .data section LMA (FLASH) and VMA (RAM) addresses:
llvm-objdump -t blinky | grep _data
60061ff0 g       *ABS*	00000000 _data_lma
61000000 g       .data	00000000 _data_vma
00000010 g       *ABS*	00000000 _data_size
---
# Extracting loadable sections to .bin file:
llvm-objcopy -O binary blinky blinky.bin
---
# Signing .bin file:
binary payload size is 7424 bytes
Signed firmware blob written to target/riscv32imac-unknown-none-elf/debug/examples/blinky.img
---
# Packing signed blob as UF2:
signed blob file size is 8192 bytes
UF2 image written to target/riscv32imac-unknown-none-elf/debug/examples/blinky.uf2
---
cp target/riscv32imac-unknown-none-elf/debug/examples/blinky.uf2 examples/
```


### examples/uart.rs

See comments in [examples/uart.rs](examples/uart.rs) for more details on what
this does.

```
$ make uart
cargo clean
     Removed 43 files, 17.7MiB total
cargo build --example uart
   Compiling dabao-sdk v0.1.0 (/home/sam/code/dabao-sdk)
    Finished `dev` profile [optimized] target(s) in 0.25s
objdump -h target/riscv32imac-unknown-none-elf/debug/examples/uart

target/riscv32imac-unknown-none-elf/debug/examples/uart:     file format elf32-little

Sections:
Idx Name          Size      VMA       LMA       File off  Algn
  0 .firmware     00002410  60060300  60060300  00000300  2**2
                  CONTENTS, ALLOC, LOAD, READONLY, CODE
  1 .data         00000010  61000000  60062710  00003000  2**2
                  CONTENTS, ALLOC, LOAD, DATA
  2 .bss          00000020  61000010  61000010  00003010  2**2
                  ALLOC
---
# Checking .data section LMA (FLASH) and VMA (RAM) addresses:
llvm-objdump -t uart | grep _data
60062710 g       *ABS*	00000000 _data_lma
61000000 g       .data	00000000 _data_vma
00000010 g       *ABS*	00000000 _data_size
---
# Extracting loadable sections to .bin file:
llvm-objcopy -O binary uart uart.bin
---
# Signing .bin file:
binary payload size is 9248 bytes
Signed firmware blob written to target/riscv32imac-unknown-none-elf/debug/examples/uart.img
---
# Packing signed blob as UF2:
signed blob file size is 10016 bytes
UF2 image written to target/riscv32imac-unknown-none-elf/debug/examples/uart.uf2
---
cp target/riscv32imac-unknown-none-elf/debug/examples/uart.uf2 examples/
```


## Pinout & Electrical Ratings

- Bunnie says the GPIO uses a 22 nm cell that can do **3.3V** IO with **12mA**
  current drive and **2kV HBM** ESD protection.

- GPIO outputs **can do Pull-Up**, but **pull-down is not supported**.

- ️⚡️🔥☠️ **DO NOT USE 5V**. IO is not 5V tolerant.

- Dabao v3 schematic:
  [github.com/baochip/dabao/blob/main/dabao_v3c.pdf](https://github.com/baochip/dabao/blob/main/dabao_v3c.pdf)

- Bootloader (boot1) serial console: **TX=PB14**, **RX=PB13**, 1M baud 8N1<br>
  (source [README-baochip.md](https://github.com/betrusted-io/xous-core/blob/main/README-baochip.md))


## Division and Precision Timing

Notes from Bunnie on relative cost of math operations on the main CPU:
- Division takes 34 cycles
- Shifts use a barrel shifter, but there's a potential for data dependencies to
  slow the pipeline. So, shift may take 1 or 2 cycles.

Tracking time is really easy with TICKTIMER if you only need a 1 ms tick for
the counter. But, it's potentially desirable to have separate µs and ms time
functions available for fine and coarse resolution timing.

1. If you use a 1 MHz tick, getting 1 µs timer resolution 64-bit counter is
   just a matter of two register reads, a shift, and an or. But, to get a 1 ms
   timestamp, you'd need to do a `/ 1000`. Division is slow, so it would be
   nice to avoid that overhead.

2. If you use a 1/1024 ms tick (1.024 MHz), you can calculate ms as
   ```
   let ms = tick >> 10;
   ```
   and µs as
   ```
   let us = (tick * 1000) >> 10;   // or simplify fraction: (tick * 125) >> 7
   ```
   In both cases, you can avoid the division. You can also use ticks directly
   if you don't need the time of precise delays to be specified in SI units.


## UART Serial Console

For UART serial debug output, you need a serial monitor that can do 1Mbaud. On
Linux, this is easy. But, on macOS, you need a serial monitor that uses the
IOSSIOSPEED ioctl to set non-standard baud rates (`screen` won't work).
Homebrew's [picocom](https://formulae.brew.sh/formula/picocom) reportedly
works. My [hbaud](https://github.com/samblenny/hbaud) works. The paid "Serial"
app on the App Store might work. NOTE: This is for UART serial. USB CDC is
different.

To read debug serial port log messages:

1. Solder 2.54mm header pins to your Dabao so you can put it in a breadboard.

2. Find a fast USB serial adapter that supports 1 Mbps. Adapters using FTDI,
   CP2102, or CP2102N chips are a good bet. The Raspberry Pi Debug Probe seems
   to work.

3. Wire up your serial adapter:

   | FTDI Adapter | Dabao     |
   | ------------ | --------- |
   | TX           | PB13 (RX) |
   | RX           | PB14 (TX) |
   | GND          | GND       |

   | Pi Debug Probe   | Dabao     |
   | ---------------- | --------- |
   | Orange Wire (TX) | PB13 (RX) |
   | Yellow Wire (RX) | PB14 (TX) |
   | Black Wire       | GND       |

4. Connect the adapter, find its device with `ls /dev/tty*` or whatever, then
   start a serial monitor with `screen -fn $ADAPTER_TTY 1000000` or whatever.

5. Connect Dabao to USB. You should see `boot0 console up` and so on.


### Boot Messages Example: USB Host

A typical boot when plugged into a USB host (computer) might look like this:

```
boot0 console up

~~boot0 up! (v0.9.16-1881-g3e4b0b657)~~

boot0 console up

~~boot0 up! (v0.9.16-1881-g3e4b0b657)~~

boot1 udma console up, CPU @ 350MHz!

~~Boot1 up! (v0.9.16-2542-g944a8082e: Towards Beta-0)~~

Configured board type: Dabao
Boot bypassed because bootwait was enabled
USB device ready
USB is connected!
```

Note the `USB is connected!` on the last line.


### Boot Messages Example: USB Power Only

A typical boot when only connected to USB power might look like this:
```
boot0 console up

~~boot0 up! (v0.9.16-1881-g3e4b0b657)~~

boot0 console up

~~boot0 up! (v0.9.16-1881-g3e4b0b657)~~

boot1 udma console up, CPU @ 350MHz!

~~Boot1 up! (v0.9.16-2542-g944a8082e: Towards Beta-0)~~

Configured board type: Dabao
Boot bypassed because bootwait was enabled
USB device ready
```

Note the absence of `USB is connected!` for the last line. Here, the bootloader
shell is bound to the UART port. If you press the enter key, you should see
something like:

```

Command not recognized:
Commands include: reset, echo, altboot, boot, bootwait, idmode, localecho, uf2, boardtype, audit, lockdown, paranoid, self_destruct
```

Then, if you type `audit` + enter, you should see something like this:

```
audit
Board type reads as: Dabao
Boot partition is: Ok(PrimaryPartition)
Semver is: v0.9.16-2542-g944a8082e
Description is: Towards Beta-0
Device serializer: xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx
Public serial number: xxxxxx
UUID: xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx
Paranoid mode: 0/0
Possible attack attempts: 0
Revocations:
Stage       key0     key1     key2     key3
boot0       enabled  enabled  enabled  enabled
boot1       enabled  enabled  enabled  enabled
next stage  enabled  enabled  enabled  enabled
Boot0: key 1/1 (bao2) -> 60000000
Boot1: key 3/3 (dev ) -> 60020000
Next stage: key 3/3 (dev ) -> 60060000
== BOOT1 FAILED PUBKEY CHECK ==
== IN DEVELOPER MODE ==
== BOOT1 REPORTED PUBKEY CHECK FAILURE ==
In-system keys have been generated
** System did not meet minimum requirements for security **
```


## USB CDC Serial Console

The Baochip bootloader and baremetal app both provide a simple shell interface.
The boot log messages always go to the UART serial port, but the shell binding
depends on whether the USB port is connected to power only or to a USB host:

- Connected to Power Only: Shell binding goes to UART serial port. In this
  case, the last UART log line is `USB device ready` (not `...connected!`).

- Connected to Computer (USB host): Shell binding goes to USB CDC serial port.
  In this case, the last UART log line should be `USB is connected!`.


## Docs, Refs, and Downloads


### Dabao Board (Bao1x dev board)

- ⚠️ The first batch of boards (e.g. from 39C3) shipped with alpha firmware
  that must be updated.

  **Dabao Bootloader Update Instructions**:
  [xous-core/README-baochip.md](https://github.com/betrusted-io/xous-core/blob/main/README-baochip.md)

  The bootloader update instructions describe building `bao1x-alt-boot1.uf2`
  and `bao1x-boot1.uf2` from source, but you can also get them from bunnie's CI
  builds at
  [ci.betrusted.io/latest-ci/baochip/bootloader/](https://ci.betrusted.io/latest-ci/baochip/bootloader/)

- Board design files: [github.com/baochip/dabao](https://github.com/baochip/dabao)


### Bao1x Chip (CSP Package)

These notes and documentation links should help understand peripheral IP blocks
used in the Bao1x SoC. Much of this came from the baochip discord server's
not-rust channel.

- Baochip website with various docs links: [baochip.com](https://baochip.com)

- uDMA block is from Pulp platform from University of Bologna/ETH Zurich:<br>
  [docs](https://docs.openhwgroup.org/projects/core-v-mcu/doc-src/udma_subsystem.html),
  [sample drivers](https://github.com/pulp-platform/pulp-rt/tree/master/drivers)

  uDMA provides UART, I2C, Camera, SPI, SDIO, and ADC peripheral access by DMA

- IRQArray is meant to be an alternative to NVIC that will work better with
  virtual memory. IRQs are split into banks of 16 with each bank getting a
  separate page of memory. This allows drivers to have their own memory spaces.

- CPU cluster register group: [docs](https://ci.betrusted.io/bao1x-cpu/)

  These RV32 registers provide access to IRQArray, timers, and suspend/resume
  features.

- Peripheral cluster register group: [docs](https://ci.betrusted.io/bao1x/)

  These RV32 registers provide access to math/crypto accelerators, TRNG, UDMA,
  BIO, and a variety of other as yet somewhat mysterious peripherals.

- PLL programming is not well documented yet. Best reference is the
  [rust source](https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/clocks.rs).
  But, the bootloader sets up the clocks and the UART, so this might not matter
  unless you need to do something like adjust the main clock to hit a specific
  I2S bit clock rate.

- Bare metal linker file:<br>
  [xous-core/baremetal/src/platform/bao1x/link.x](https://github.com/betrusted-io/xous-core/blob/main/baremetal/src/platform/bao1x/link.x)

- Bare metal stack pointer, trap, and entry point setup:<br>
  [xous-core/baremetal/src/asm.rs](https://github.com/betrusted-io/xous-core/blob/944a8082ec235339e5e73165da48fd209f4a0724/baremetal/src/asm.rs#L35-L56)

- Existing BIO examples depend on Xous APIs. There isn't much documentation yet
  on how to use it outside of Xous (as of Jan 10, 2026). BIO is relevant for
  handling timing sensitive IO like I2S, 1-wire, or LED string drivers.

  Current no_std BIO drivers:
  [xous-core/libs/bao1x-hal/src/bio_hw.rs](https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/bio_hw.rs)


## Notes from Xous Docs

These have some useful info for dev environment setup along with building,
signing, and flashing apps for Baochip:

1. https://betrusted.io/xous-book/ch01-02-hello-world.html
2. https://github.com/betrusted-io/xous-core/blob/main/README-baochip.md


## Binary Signing and UF2 Notes

The bao1x bootloader expects binary images to be ed25519ph signed with  using
the developer key that's available at
[betrusted-io/xous-core/devkey/dev.key](https://github.com/betrusted-io/xous-core/blob/main/devkey/dev.key).

The xous-core repo includes tools to go from ELF binary, to signed ELF binary,
to bootloader-ready UF2 file. These scripts and tools from xous-core are
involved in signing and UF2 creation:

- Windows PowerShell script to sign and deploy official build artifacts for
  Dabao using a hardware signing token:<br>
  [xous-core/baosign.ps1](https://github.com/betrusted-io/xous-core/blob/main/baosign.ps1)

- This tool converts .img pre-sign binaries (derived from ELF binaries) into
  UF2 encoded signed binaries. There's some moderately complicated stuff going
  on with wrapping the object code in a file structure that includes public
  keys, padding, version info, the signature, and maybe a couple other
  things.<br>
  [xous-core/tools/src/sign_image.rs](https://github.com/betrusted-io/xous-core/blob/main/tools/src/sign_image.rs) (library functions)<br>
[xous-core/tools/src/bin/sign_image.rs](https://github.com/betrusted-io/xous-core/blob/main/tools/src/bin/sign_image.rs) (command)

  The public keys that get embedded in the signed output image come from:<br>
  [xous-core/libs/bao1x-api/src/*.rs](https://github.com/betrusted-io/xous-core/tree/main/libs/bao1x-api/src/pubkeys)

  The developer key private key PEM comes from:<br>
  [xous-core/devkey/dev.key](https://github.com/betrusted-io/xous-core/blob/main/devkey/dev.key)

  The developer key public key certificate PEM comes from:<br>
  [xous-core/devkey/dev-x509.crt](https://github.com/betrusted-io/xous-core/blob/main/devkey/dev-x509.crt)


- This section of the Xous `xtask` tool shows the command line arguments that
  it uses when invoking `sign_image.rs` to sign a baremetal binary:<br>
  [xous-core/xtask/src/builder.rs#L978-L1000](https://github.com/betrusted-io/xous-core/blob/32c5d492cdd745f2f36163564025a9a93c90422a/xtask/src/builder.rs#L978-L1000)

  That ends up doing the equivalent of:

   ```
    target/debug/sign-image --loader-image \
    target/riscv32imac-unknown-none-elf/release/baremetal-presign.img \
    --loader-key devkey/dev.key --loader-output \
    target/riscv32imac-unknown-none-elf/release/baremetal.img \
    --min-xous-ver v0.9.8-791 --sig-length 768 --with-jump --bao1x \
    --function-code baremetal
   ```

- This will convert an ELF file to a pre-sign object (.img file):

   ```
    target/debug/copy-object \
    xous-core/target/riscv32imac-unknown-none-elf/release/baremetal \
    target/riscv32imac-unknown-none-elf/release/baremetal-presign.img --bao1x
   ```

  You can get a help message for `copy-object` from xous-core by doing:

  ```
  cargo run --package tools --bin copy-object
  ```


## Build & Run sign_image.rs

1. Install rust with rustup

2. Clone xous-core:
   ```
   git clone --depth 100 https://github.com/betrusted-io/xous-core.git
   ```

3. Build sign-image:
   ```
   cargo run -p tools --bin sign-image
   ```

4. Run it:
   ```
   target/debug/sign-image --help
   ```


## Alternate Signing Method

My [signer.py](signer.py) script implements a functionally equivalent signing
operation to `sign-image` invoked with the baremetal bao1x options listed
above. The point of my signer script is to make it possible to sign without
pulling in xous-core as a dependency.

The signer requires that you convert the ELF file to a binary blob of machine
code in the style of `copy-object`. My Makefile does the ELF to machine code
extraction step using `llvm-objcopy`, with some help from the linker script.


## Understanding the Blob Format and Early Boot

The pre-sign image created by `copy-object` is meant to be copied to RRAM (the
Baochip equivalent of flash) for XIP (execute in place) access by the CPU. The
blob contains:

1. A header, beginning with a jump instruction, that's sufficient to
   reconstruct statics from the `.data` section. This is a method to compress
   `.data`, which typically has many zero bytes. This matters particularly for
   the bootloaders, which are space constrained to fit in small ReRAM slots.

2. The `.text` section with executable code (assembly and compiled rust code).
   This begins with init code to set up hardware, prepare the `.data` section
   in SRAM, zero the `.bss` section in SRAM, configure interrupt and trap
   handlers, set the stack pointer, and jump to `_start`.

3. The `.rodata` section with read-only data that stays in ReRAM (flash).

For my purposes, compressing `.data` is probably not necessary. But, my code in
`.text` will still be responsible for initializing `.data` and `.bss` by some
means.

If using `objcopy` to create a blob instead of `copy-object`, it will be
important to ensure section start offsets within the blob file stay consistent
with the LMA addresses in the ELF binary. This may involve padding. Making sure
the blob gets written to the correct RRAM start address happens by setting the
offset in the UF2 file (see `signer.py` or `sign_image.rs`).

For link script info and `.data` initialization details, see:

- [xous-core/baremetal/src/platform/bao1x/link.x](https://github.com/betrusted-io/xous-core/blob/main/baremetal/src/platform/bao1x/link.x)
- [xous-core/baremetal/src/platform/bao1x/bao1x.rs](https://github.com/betrusted-io/xous-core/blob/d26ce7fbf11fef8aac24adea93f557341dd0600f/baremetal/src/platform/bao1x/bao1x.rs#L52-L72)
- [xous-core/tools/src/bin/copy-object.rs](https://github.com/betrusted-io/xous-core/blob/d26ce7fbf11fef8aac24adea93f557341dd0600f/tools/src/bin/copy-object.rs#L55-L84)

For early hardware setup, including IRQs, see:

- [xous-core/baremetal/src/platform/bao1x/irq.rs](https://github.com/betrusted-io/xous-core/blob/d26ce7fbf11fef8aac24adea93f557341dd0600f/baremetal/src/platform/bao1x/irq.rs#L10-L28)

Bunnie says, when the Bao1x bootloader jumps to the initial JAL instruction at
the start of the signed blob in ReRAM, interrupts are guaranteed to be off.
Also, at that point, the UDMA UART baud rate, buffers, and clocks will be set
up and ready to use (but it's best to re-initialize them anyway).


## UDMA Notes

Some relevant UDMA UART code snippets from xous-core:

- TX usage (`putc` definition):
  [xous-core/libs/bao1x-hal/src/debug.rs#L12-L26](https://github.com/betrusted-io/xous-core/blob/5eec0702f9a989144739ff08d419ed7445c2ecc9/libs/bao1x-hal/src/debug.rs#L12-L26)

- `get_handle` definition:
  [xous-core/libs/bao1x-hal/src/udma/uart.rs#L110-L125](https://github.com/betrusted-io/xous-core/blob/5eec0702f9a989144739ff08d419ed7445c2ecc9/libs/bao1x-hal/src/udma/uart.rs#L110-L125)

- `write` definition:
  [xous-core/libs/bao1x-hal/src/udma/uart.rs#L184-L224](https://github.com/betrusted-io/xous-core/blob/5eec0702f9a989144739ff08d419ed7445c2ecc9/libs/bao1x-hal/src/udma/uart.rs#L184-L224)

- `udma_enqueue` definition:
  [xous-core/libs/bao1x-hal/src/udma/mod.rs#L253-L274](https://github.com/betrusted-io/xous-core/blob/main/libs/bao1x-hal/src/udma/mod.rs#L253-L274)

**CAUTION:** The UDMA engine expects its source data to come from the IFRAM
buffers, **which are outside the regular SRAM**. The IFRAM address space is a
totally different thing, a 256kB region of buffers mapped to its own address
range starting at 0x50000000.

IFRAM address details:

```
pub const HW_IFRAM0_MEM:     usize = 0x50000000;
pub const HW_IFRAM0_MEM_LEN: usize = 131072;  // 128 kB
pub const HW_IFRAM1_MEM:     usize = 0x50020000;
pub const HW_IFRAM1_MEM_LEN: usize = 131072;  // 128 kB
```

The only way to send anything on the UDMA UART is to set up a DMA transaction.
So, you have to set up an `unsafe` buffer in IFRAM, copy your data there, then
start a UDMA transaction to read from the buffer.
