<!-- SPDX-License-Identifier: MIT -->
<!-- SPDX-FileCopyrightText: Copyright 2026 Sam Blenny -->
# Dabao Zephyr CircuitPython PoC

**DRAFT: WORK IN PROGRESS**

[*an aspirational blurb about what this is trying to be*:]

This is a proof of concept for building CircuitPython's Zephyr port to run on
the Baochip Dabao board. Most of the work here is about getting Rust no_std
device drivers from the Dabao bootloader to work as Zephyr drivers. There's
some additional stuff for getting the CircuitPython Zephyr port to build with
my Zephyr board definition for Dabao.

**CAUTION:**

1. This is a proof of concept. I don't currently have any intention to maintain
   this once I've got it working. The point is to document how to do it.

2. The Zephyr Project has strict contribution policies which I am not
   following. Commits in this repo are unsuitable for upstreaming. But...

3. I'm arranging things here to hopefully facilitate easy re-implementation or
   translation of my work (e.g. Rust to C with a new copyright). If you want to
   do that and follow the Zephyr Project upstreaming policies, go for it. Best
   of luck.


## Goals & Strategy

What I care about here is getting to the point of CircuitPython running on a
Dabao board by the most straightforward path which seems reasonable and in good
taste. Plan is CPU, ReRAM, RAM, LED blinky, and UART serial first, then maybe
add USB.

Zephyr's configuration system is more complex than I expect to be useful, so I
plan to bypass some of it if I can. Specifically, if possible, I hope to use
the Rust drivers with a C FFI wrapper. To the extent that using Zephyr's
typical Device Tree macros gets in the way of using the Rust drivers, I hope to
bypass the Device Tree stuff.


## Pinout & Electrical Ratings

- Bunnie says the GPIO uses a 22 nm cell that can do **3.3V** IO with **12mA**
  current drive and **2kV HBM** ESD protection.

- ️⚡️🔥☠️ **DO NOT USE 5V**. IO is not 5V tolerant.

- Dabao v3 schematic:
  [github.com/baochip/dabao/blob/main/dabao_v3c.pdf](https://github.com/baochip/dabao/blob/main/dabao_v3c.pdf)

- Bootloader (boot1) serial console: **TX=PB14**, **RX=PB13**, 1M baud 8N1<br>
  (source [README-baochip.md](https://github.com/betrusted-io/xous-core/blob/main/README-baochip.md))


## UART Serial Console

⚠️ Using **macOS** `screen -fn /dev/tty.usb... 1000000` to get a 1 Mbps serial
monitor **won't work** because macOS screen doesn't know how to set
non-standard baud rates. The **easy fix is use Linux**. If you REALLY want to
use macOS, you'll need to find a program that can use the IOSSIOSPEED ioctl to
set non-standard baud rates. The paid "Serial" app on the macOS App Store might
work. NOTE: This is only an issue for UART serial. USB CDC serial is different.

You might wonder, why 1M instead of 115200 or whatever? Some benefits:

- Faster baud rate means faster boot times without sacrificing logging detail.

- 1M is easy to derive with low error by dividing down the system clock, while
  legacy baud rates require PLL scaling that gets jittery as you go faster.

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

The bootloader and baremetal app both provide a simple shell interface. The
boot log messages always go to the UART serial port, but the shell binding
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


### CircuitPython

- [circuitpython/ports/zephyr-cp/README.md](https://github.com/adafruit/circuitpython/blob/main/ports/zephyr-cp/README.md)
  explains how to install and build the CircuitPython Zephyr port

- CircuitPython Zephyr port build tools:
  [ports/zephyr-cp/cptools](https://github.com/adafruit/circuitpython/tree/main/ports/zephyr-cp/cptools)

  See [zephyr2cp.py](https://github.com/adafruit/circuitpython/blob/main/ports/zephyr-cp/cptools/zephyr2cp.py)
for details of how CircuitPython extracts pin mappings from Device Tree config)


### Zephyr

These are for implementing Zephyr drivers, board definitions, etc.

- [How to Build Drivers for Zephyr RTOS](https://www.zephyrproject.org/how-to-build-drivers-for-zephyr-rtos/)
  (Zephyr Project blog, August 11, 2020)

- [LiteX VexRiscv](https://docs.zephyrproject.org/latest/boards/enjoydigital/litex_vexriscv/doc/index.html)
  board definition

- UART driver samples (github):
  [zephyr/samples/uart/](https://github.com/zephyrproject-rtos/zephyr/tree/main/samples/drivers/uart)

- Zephyr Project
  [UART API docs](https://docs.zephyrproject.org/latest/hardware/peripherals/uart.html)

- Zephyr Project
  [GPIO API docs](https://docs.zephyrproject.org/latest/hardware/peripherals/gpio.html)


## Notes from Xous Docs

These have some useful info for dev environment setup along with building,
signing, and flashing apps for Baochip:

1. https://betrusted.io/xous-book/ch01-02-hello-world.html
2. https://github.com/betrusted-io/xous-core/blob/main/README-baochip.md


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

**TODO: Finish this**


## Binary Signing and UF2 Notes

The bao1x bootloader expects binary images to be ed25519ph signed with  using
the developer key that's available at
[betrusted-io/xous-core/devkey/dev.key](https://github.com/betrusted-io/xous-core/blob/main/devkey/dev.key).

The xous-core repo includes tools to go from ELF binary, to signed ELF binary,
to bootloader-ready UF2 file. I'll need to implement an equivalent of that
workflow for C binaries that link to Rust drivers through an FFI wrapper.
Ideally, I'd like tooling that works without a dependency on cloning the
xous-core repo or invoking `cargo`. Definitely I want to avoid using `xtask`.

These scripts and tools from xous-core are involved in signing and UF2
creation:

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

The signer still requires that you convert the ELF file to a binary blob of
object code in the style of `copy-object`. But, it would probably also work to
use gcc's `objcopy` tool. (I haven't tried that yet.)


## Understanding the Blob Format and Early Boot

The pre-sign image created by `copy-object` is meant to be copied to RRAM (the
Baochip equivalent of flash) for XIP (execute in place) access by the CPU. The
blob contains:

1. A header, beginning with a jump instruction, that's sufficient to
   reconstruct statics from the `.data` section. This is a method to compress
   `.data`, which typically has many zero bytes. This matters particularly for
   the bootloaders, which are space constrained to fit in small RRAM slots.

2. The `.text` section with executable code (assembly and compiled rust code).
   This begins with init code to set up hardware, prepare the `.data` section
   in RAM, zero the `.bss` section in RAM, configure interrupt and trap
   handlers, set the stack pointer, and jump to `_start`.

3. The `.rodata` section with read-only data that stays in RRAM (flash).

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
the start of the signed blob in RRAM, interrupts are guaranteed to be off.
Also, at that point, the UDMA UART baud rate, buffers, and clocks will be set
up and ready to use (but it's best to re-initialize them anyway).

Some relevant UDMA UART code snippets from xous-core:

- TX usage:

  ```
  let mut udma_uart = unsafe {
      // safety: this is safe to call, because we set up clock and events prior
      // to calling new.
      udma::Uart::get_handle(
          utra::udma_uart_2::HW_UDMA_UART_2_BASE,
          uart_buf_addr,
          uart_buf_addr,
      )
  };
  udma_uart.write(&buf);
  ```

- `get_handle` definition:

  ```
  pub unsafe fn get_handle(csr_virt_addr: usize, udma_phys_addr: usize,
  udma_virt_addr: usize) -> Self {
      assert!(UART_RX_BUF_SIZE + UART_TX_BUF_SIZE == 4096,
          "Configuration error in UDMA UART");
      let csr = CSR::new(csr_virt_addr as *mut u32);
      Uart {
          csr,
          ifram: IframRange::from_raw_parts(
              udma_phys_addr,
              udma_virt_addr,
              UART_RX_BUF_SIZE + UART_TX_BUF_SIZE,
          ),
      }
  }
  ```

- `write` definition:

  ```
  pub fn write(&mut self, buf: &[u8]) -> usize {
      let mut writelen = 0;
      for chunk in buf.chunks(UART_TX_BUF_SIZE) {
              self.ifram.as_slice_mut()[..chunk.len()].copy_from_slice(chunk);
              unsafe {
                  self.udma_enqueue(
                      Bank::Tx,
                      &self.ifram.as_phys_slice::<u8>()[..chunk.len()],
                      CFG_EN | CFG_SIZE_8,
                  );
                  writelen += chunk.len();
              }
          self.wait_tx_done();
      }
      writelen
  }
  ```

- `udma_enqueue` definition:

  ```
  unsafe fn udma_enqueue<T>(&self, bank: Bank, buf: &[T], config: u32) {
      let bank_addr = self.csr().base().add(bank as usize);
      let buf_addr = buf.as_ptr() as u32;
      bank_addr.add(DmaReg::Saddr.into()).write_volatile(buf_addr);
      bank_addr.add(DmaReg::Size.into()).write_volatile(
          (buf.len() * size_of::<T>()) as u32);
      bank_addr.add(DmaReg::Cfg.into()).write_volatile(
          config | CFG_EN | CFG_BACKPRESSURE)
  }
  ```

**CAUTION:** The UDMA engine expects its source data to come from the IFRAM
buffers, **which are outside the regular RAM**. The IFRAM address space is a
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
